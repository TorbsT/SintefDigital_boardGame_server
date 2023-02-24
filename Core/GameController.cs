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
    private readonly List<GameStateInfo> _games;
    private readonly IMultiPlayerInfoViewController _viewController;
    private readonly IMultiPlayerInfoPlayerInfoInputController _inputController;
    private readonly ILogger _logger;
    private Thread _mainLoopThread;
    private object _stopMonitor = new object();
    private bool _stopMainThread = false;

    public GameController(ILogger logger, IMultiPlayerInfoViewController viewController, IMultiPlayerInfoPlayerInfoInputController inputController)
    {
        this._games = new List<GameStateInfo>();
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
        lock (_stopMonitor)
        {
            stop = _stopMainThread;
        }

        while (!stop)
        {
            try
            {
                List<(PlayerInfo, string)> newGames = new List<(PlayerInfo, string)>();
                try
                {
                    _inputController.Lock();
                    newGames = _inputController.FetchRequestedGameLobbiesWithLobbyNameAndPlayerInfo();
                }
                catch (Exception e)
                {
                    _logger.Log(LogLevel.Error, $"Failed to fetch new game lobbies. Error {e}");
                }
                finally
                {
                    _inputController.ReleaseLock();
                }
                foreach (var lobbyNameAndPlayerInfo in newGames) HandleNewGameCreation(lobbyNameAndPlayerInfo);
            }
            catch (Exception e)
            {
                _logger.Log(LogLevel.Error, $"Failed to create new game(s). Error {e}.");

            }

            try
            {
                HandlePlayerInfoInputs();
            }
            catch (Exception e)
            {
                _logger.Log(LogLevel.Error, $"Failed to handle PlayerInfo inputs {e}.");
            }

            lock (_stopMonitor)
            {
                stop = _stopMainThread;
            }
        }
    }

    private void HandleNewGameCreation((PlayerInfo, string) lobbyNameAndPlayerInfo)
    {
        var newGame = CreateNewGameAndAssignHost(lobbyNameAndPlayerInfo);
        _games.Add(newGame);
        try
        {
            _viewController.Lock();
            _viewController.SendNewGameStateInfoToPlayerInfos(newGame);
        }
        catch (Exception e)
        {
            _logger.Log(LogLevel.Error, "Something went wrong when trying to send new game state to the PlayerInfos." +
                                        $" Error: {e}");
        }
        finally
        {
            _viewController.ReleaseLock();
        }
    }

    private GameStateInfo CreateNewGameAndAssignHost((PlayerInfo, string) lobbyNameAndPlayerInfo)
    {
        _logger.Log(LogLevel.Debug, "Creating new game state.");
        var newGame = new GameStateInfo
        {
            ID = GenerateUnusedGameID(),
            Name = lobbyNameAndPlayerInfo.Item2,
            PlayerInfos = new List<PlayerInfo>()
        };
        PlayerInfo PlayerInfo = lobbyNameAndPlayerInfo.Item1;
        AssignGameToPlayerInfo(ref PlayerInfo, newGame);
        newGame.PlayerInfos.Add(PlayerInfo);
        _logger.Log(LogLevel.Debug, $"Done creating new Game State with ID {newGame.ID} and name {newGame.Name}.");
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
            if (game.ID == ID)
            {
                return false;
            }
        }
        return true;
    }

    private void AssignGameToPlayerInfo(ref PlayerInfo PlayerInfo, GameStateInfo game)
    {
        PlayerInfo.ConnectedGameID = game.ID;
        _logger.Log(LogLevel.Debug, $"Assigned PlayerInfo with uniqueID {PlayerInfo.UniqueID} to game with id {game.ID}");
    }

    private void HandlePlayerInfoInputs()
    {
        foreach (var game in _games)
        {
            List<Input> PlayerInfoInputs = new List<Input>();
            try
            {
                _inputController.Lock();
                PlayerInfoInputs = _inputController.FetchPlayerInfoInputs(game.ID);
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
                    var newGameStateInfo = HandleInput(input);
                    _logger.Log(LogLevel.Debug, $"Got new game state with ID {newGameStateInfo.ID}.");
                    if (newGameStateInfo.Equals(default)) continue;
                    _viewController.Lock();
                    _viewController.SendNewGameStateInfoToPlayerInfos(newGameStateInfo);
                    _logger.Log(LogLevel.Debug, $"Done sending new game state to PlayerInfos in game with ID " +
                                                $"{newGameStateInfo.ID}");
                    _viewController.ReleaseLock();
                }
                catch (Exception e)
                {
                    _logger.Log(LogLevel.Error, $"Failed to handle input: {input.ToString()}. Error {e}");
                }
            }
        }
    }

    private GameStateInfo HandleInput(Input input)
    {
        // TODO check if input is legal based on the game state once applicable
        _logger.Log(LogLevel.Debug, $"Handling inputs for PlayerInfo with uniqueID {input.PlayerInfo.UniqueID} and " +
                                                   $"name {input.PlayerInfo.Name} and input type {input.Type}.");
        switch (input.Type)
        {
            case PlayerInfoInputType.Movement:
                return HandleMovement(input.PlayerInfo, input.RelatedNode);
            default:
                throw new ArgumentOutOfRangeException();
        }
        _logger.Log(LogLevel.Debug, $"Finished handling inputs for PlayerInfo with ID {input.PlayerInfo.UniqueID}.");
    }

    private GameStateInfo HandleMovement(PlayerInfo PlayerInfoCopy, NodeInfo toNodeCopy)
    {
        try
        {
            var game = _games.First(state => state.ID == PlayerInfoCopy.ConnectedGameID);
            var gamePlayerInfo = game.PlayerInfos.First(PlayerInfo1 => PlayerInfo1.InGameID == PlayerInfoCopy.InGameID);
            _games.Remove(game);
            game.PlayerInfos.RemoveAll(PlayerInfo1 => PlayerInfo1.InGameID == PlayerInfoCopy.InGameID);
            // TODO: Check here if the movement is valid once applicable and dont use toNodeCopy.
            gamePlayerInfo.Position = toNodeCopy;
            game.PlayerInfos.Add(gamePlayerInfo);
            _games.Add(game);
            _logger.Log(LogLevel.Debug, $"Moved PlayerInfo {PlayerInfoCopy.InGameID} in {PlayerInfoCopy.ConnectedGameID} to " +
                                        $"node with nodeID {toNodeCopy.ID}");
            return game;
        }
        catch (InvalidOperationException e)
        {
            _logger.Log(LogLevel.Error, "Failed to move PlayerInfo because the game the PlayerInfo refers to " +
                                        $"doesn't exist or the PlayerInfo isn't in the game. " +
                                        $"GameID: {PlayerInfoCopy.ConnectedGameID}. InGame PlayerInfoID {PlayerInfoCopy.InGameID}.");
        }
        catch (Exception e)
        {
            _logger.Log(LogLevel.Error, $"Something went wrong when trying to move the PlayerInfo. Error {e}");
        }

        return default;
    }


    public void Dispose()
    {
        Stop();
        _mainLoopThread.Join();
    }
}