using Logging;
using System;
using System.Collections.Generic;
using System.Security.Principal;
using System.Threading;


namespace Core;

/// <summary>
/// Remember to Dispose the GameController when stopping the application!
/// </summary>
public class GameController : IDisposable
{
    private readonly List<GameState> _games;
    private readonly IMultiPlayerInfoViewController _viewController;
    private readonly IMultiPlayerInfoPlayerInfoInputController _inputController;
    private readonly ILogger _logger;
    private Thread _mainLoopThread;
    private object _stopMonitor = new object();
    private bool _stopMainThread = false;
    private List<int> _uniqueIDs= new List<int>();

    public GameController(ILogger logger, IMultiPlayerInfoViewController viewController, IMultiPlayerInfoPlayerInfoInputController inputController)
    {
        this._games = new List<GameState>();
        this._viewController = viewController;
        this._inputController = inputController;
        this._logger = logger;
        this._mainLoopThread = new Thread(RunMainLoop);
    }

    //TODO: Make an instantiator that gives a GameController that is threadsafe.

    public void Run()
    {
        if (IsMainLoopRunning())
        {
            _logger.Log(LogLevel.Error, "The GameController is already running!");
            return;
        }

        _mainLoopThread.Start();
    }

    public bool IsMainLoopRunning()
    {
        return _mainLoopThread.IsAlive;
    }

    public void Stop()
    {
        lock (_stopMonitor)
        {
            _stopMainThread = true;
        }
    }

    private void RunMainLoop()
    {
        bool stop;
        lock (_stopMonitor) {stop = _stopMainThread;}

        while (!stop)
        {
            try
            {
                int amountOfNewIDs = 0;
                try
                {
                    _inputController.Lock();
                    amountOfNewIDs = _inputController.FetchWantedAmountOfUniqueIDs();
                } 
                catch (Exception e) {_logger.Log(LogLevel.Error, $"Failed to fetch amount of wanted IDs. Error {e}");}
                finally {_inputController.ReleaseLock();}

                HandleNewIDs(amountOfNewIDs);

            } catch (Exception e)
            {
                _logger.Log(LogLevel.Error, $"Failed to make new unique IDs. Error: {e}");
            }


            List<(PlayerInfo, string)> newGames = new List<(PlayerInfo, string)>();
            try
            {
                _inputController.Lock();
                newGames = _inputController.FetchRequestedGameLobbiesWithLobbyNameAndPlayerInfo();
            }
            catch (Exception e) { _logger.Log(LogLevel.Error, $"Failed to fetch new game lobbies. Error {e}"); }
            finally { _inputController.ReleaseLock(); }

            foreach (var lobbyNameAndPlayerInfo in newGames)
            {
                _logger.Log(LogLevel.Debug, $"New games to create {newGames.Count.ToString()}. Next playerinfo {lobbyNameAndPlayerInfo.Item1.ToString()}");
                try { HandleNewGameCreation(lobbyNameAndPlayerInfo); }
                catch (Exception e) { _logger.Log(LogLevel.Error, $"Failed to create new game(s). Error {e}."); }
            }

            try { HandlePlayerInfoInputs(); }
            catch (Exception e) {_logger.Log(LogLevel.Error, $"Failed to handle PlayerInfo inputs {e}.");}

            lock (_stopMonitor) {stop = _stopMainThread;}
        }
    }

    private void HandleNewIDs(int amountOfNewIDs)
    {
        try
        {
            List<int> newIDs = GenerateUnusedGameIDs(amountOfNewIDs);
            _uniqueIDs.AddRange(newIDs);
            try
            {
                _viewController.Lock();
                _viewController.HandleGeneratedUniqueIDs(newIDs);
            }
            catch (Exception e)
            {
                _logger.Log(LogLevel.Error, $"Something went wrong when trying to give the viewController the new IDs. Error: {e}");
            }
            finally { _viewController.ReleaseLock(); }
        } 
        catch(Exception e)
        {
            _logger.Log(LogLevel.Error, $"Something went wrong when creating new unique IDs. Error: {e}");
        }
        
    }

    private List<int> GenerateUnusedGameIDs(int amountOfNewIDs)
    {
        Random generator = new Random();
        List<int> newIDs = new List<int>();
        for (int i = 0; i < amountOfNewIDs; i++)
        {
            int id = generator.Next();

            for (int _ = 0; _ < 100_000; _++)
            {
                if (!_uniqueIDs.Contains(id) && !newIDs.Contains(id)) break;
                id = generator.Next();
            }
            newIDs.Add(id);
        }
        if (newIDs.Count < amountOfNewIDs) throw new Exception("Failed to generate the wanted amount of new unique IDs");
        return newIDs;
    }

    private void HandleNewGameCreation((PlayerInfo, string) lobbyNameAndPlayerInfo)
    {
        var newGame = CreateNewGameAndAssignHost(lobbyNameAndPlayerInfo);
        _games.Add(newGame);
        try
        {
            _viewController.Lock();
            _viewController.SendNewGameStateInfoToPlayerInfos(newGame.GetGameStateInfo());
        }
        catch (Exception e) {
            _logger.Log(LogLevel.Error, "Something went wrong when trying to send new game state to the PlayerInfos." +
                                        $" Error: {e}");
        }
        finally {_viewController.ReleaseLock();}
    }

    private GameState CreateNewGameAndAssignHost((PlayerInfo, string) lobbyNameAndPlayerInfo)
    {
        _logger.Log(LogLevel.Debug, "Creating new game state.");
        foreach (GameState gameState in _games) if (gameState.ContainsUniquePlayerID(lobbyNameAndPlayerInfo.Item1)) throw new ArgumentException($"Player with unique ID {lobbyNameAndPlayerInfo.Item1.UniqueID} is connected to a game in progress"); //TODO: This line is causing problems for the tests
        var newGame = new GameState(lobbyNameAndPlayerInfo.Item2, GenerateUnusedGameID());
        newGame.AssignPlayerToGame(lobbyNameAndPlayerInfo.Item1);
        _logger.Log(LogLevel.Debug, $"Done creating new Game State with ID {newGame.GetGameStateInfo().ID} and name {newGame.GetGameStateInfo().Name}.");

        return newGame;
    }

    private int GenerateUnusedGameID()
    {
        var randomGenerator = new Random();
        var ID = randomGenerator.Next();
        while (!IsGameIDUnique(ID))
        {
            ID = randomGenerator.Next();
        }
        return ID;
    }

    private bool IsGameIDUnique(int ID)
    {
        foreach (var game in _games)
        {
            if (game.GetGameStateInfo().ID == ID)
            {
                return false;
            }
        }
        return true;
    }

    private void HandlePlayerInfoInputs()
    {
        foreach (var game in _games)
        {
            List<Input> PlayerInfoInputs = new List<Input>();
            try
            {
                _inputController.Lock();
                PlayerInfoInputs = _inputController.FetchPlayerInfoInputs(game.GetGameStateInfo().ID);
            }
            catch (Exception e)
            {
                _logger.Log(LogLevel.Error, "Something went wrong when trying to handle the PlayerInfo inputs. " +
                                            $"Error: {e}");
            }
            finally
            {
                _inputController.ReleaseLock();
            }

            foreach (var input in PlayerInfoInputs)
            {
                try
                {
                    HandleInput(input);
                    _viewController.Lock();
                    _viewController.SendNewGameStateInfoToPlayerInfos(_games.First(state => state.GetGameStateInfo().ID == input.PlayerInfo.ConnectedGameID));
                    _viewController.ReleaseLock();
                }
                catch (Exception e)
                {
                    _logger.Log(LogLevel.Error, $"Failed to handle input: {input.ToString()}. Error {e}");
                }
            }
        }
    }

    private void HandleInput(Input input)
    {
        // TODO check if input is legal based on the game state once applicable
        _logger.Log(LogLevel.Debug, $"Handling inputs for PlayerInfo with uniqueID {input.PlayerInfo.UniqueID} and " +
                                                   $"name {input.PlayerInfo.Name} and input type {input.Type}.");
        switch (input.Type)
        {
            case PlayerInfoInputType.Movement:
                HandleMovement(input.PlayerInfo, input.RelatedNode);
                break;
            default:
                throw new ArgumentOutOfRangeException();
        }
        _logger.Log(LogLevel.Debug, $"Finished handling inputs for PlayerInfo with ID {input.PlayerInfo.UniqueID}.");
    }

    private void HandleMovement(PlayerInfo playerInfo, NodeInfo toNodeCopy)
    {
        try
        {
            var game = _games.First(state => state.GetGameStateInfo().ID == playerInfo.ConnectedGameID);
            // TODO: Check here if the movement is valid once applicable and dont use toNodeCopy.
            playerInfo.Position = toNodeCopy;
            game.UpdatePlayersBasedOnInfos(new List<PlayerInfo>() {playerInfo});
            _logger.Log(LogLevel.Debug, $"Moved player {playerInfo.InGameID} in {playerInfo.ConnectedGameID} to " +
                                        $"node with nodeID {toNodeCopy.ID}");
        }
        catch (InvalidOperationException e)
        {
            _logger.Log(LogLevel.Error, "Failed to move PlayerInfo because the game the PlayerInfo refers to " +
                                        $"doesn't exist or the PlayerInfo isn't in the game. " +
                                        $"GameID: {playerInfo.ConnectedGameID}. InGame PlayerInfoID {playerInfo.InGameID}. Error: {e}");
        }
        catch (Exception e)
        {
            _logger.Log(LogLevel.Error, $"Something went wrong when trying to move the PlayerInfo. Error {e}");
        }
    }


    public void Dispose()
    {
        Stop();
        _mainLoopThread.Join();
    }
}