using System;
using System.Collections.Generic;
using System.Security.Principal;
using System.Threading;

using SintefDigital_boardGame_server.Logging;


namespace SintefDigital_boardGame_server.Core;

/// <summary>
/// Remember to Dispose the GameController when stopping the application!
/// </summary>
public class GameController : IDisposable
{
    private readonly List<GameState> _games;
    private readonly IMultiplayerViewController _viewController;
    private readonly IMultiplayerPlayerInputController _inputController;
    private readonly ILogger _logger;
    private Thread _mainLoopThread;
    private object _stopMonitor = new object();
    private bool _stopMainThread = false;

    public GameController(ILogger logger, IMultiplayerViewController viewController, IMultiplayerPlayerInputController inputController)
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
        lock (_stopMonitor)
        {
            stop = _stopMainThread;
        }
        
        while (!stop){
            try
            {
                List<(Player,string)> newGames = new List<(Player,string)>();
                try
                {
                    _inputController.Lock();
                    newGames = _inputController.FetchRequestedGameLobbiesWithLobbyNameAndPlayer();
                }
                catch (Exception e)
                {
                    _logger.Log(LogLevel.Error, $"Failed to fetch new game lobbies. Error {e}");
                }
                finally
                {
                    _inputController.ReleaseLock();
                }
                foreach (var lobbyNameAndPlayer in newGames) HandleNewGameCreation(lobbyNameAndPlayer);
            }
            catch (Exception e)
            {
                _logger.Log(LogLevel.Error, $"Failed to create new game(s). Error {e}.");
                
            }
            
            try
            {
                HandlePlayerInputs();
            }
            catch (Exception e)
            {
                _logger.Log(LogLevel.Error, $"Failed to handle player inputs {e}.");
            }

            lock (_stopMonitor)
            {
                stop = _stopMainThread;
            }
        }
    }

    private void HandleNewGameCreation((Player, string) lobbyNameAndPlayer)
    {
        var newGame = CreateNewGameAndAssignHost(lobbyNameAndPlayer);
        _games.Add(newGame);
        try
        {
            _viewController.Lock();
            _viewController.SendNewGameStateToPlayers(newGame);
        }
        catch (Exception e)
        {
            _logger.Log(LogLevel.Error, "Something went wrong when trying to send new game state to the players." +
                                        $" Error: {e}");
        }
        finally
        {
            _viewController.ReleaseLock();
        }
    }
    
    private GameState CreateNewGameAndAssignHost((Player, string) lobbyNameAndPlayer)
    {
        _logger.Log(LogLevel.Debug, "Creating new game state.");
        var newGame = new GameState
        {
            ID = GenerateUnusedGameID(),
            Name = lobbyNameAndPlayer.Item2,
            Players = new List<Player>()
        };
        Player player = lobbyNameAndPlayer.Item1;
        AssignGameToPlayer(ref player, newGame);
        newGame.Players.Add(player);
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

    private void AssignGameToPlayer(ref Player player, GameState game)
    {
        player.ConnectedGameID = game.ID;
        _logger.Log(LogLevel.Debug, $"Assigned player with uniqueID {player.UniqueID} to game with id {game.ID}");
    }

    private void HandlePlayerInputs()
    {
        foreach (var game in _games)
        {
            List<Input> playerInputs = new List<Input>();
            try
            {
                _inputController.Lock();
                playerInputs = _inputController.FetchPlayerInputs(game.ID);
            }
            catch (Exception e)
            {
                _logger.Log(LogLevel.Error, "Something went wrong when trying to handle the player inputs. " +
                                            $"Error: {e}");
            }
            finally
            {
                _inputController.ReleaseLock();
            }
            
            foreach (var input in playerInputs)
            {
                try
                {
                    var newGameState = HandleInput(input);
                    _logger.Log(LogLevel.Debug, $"Got new game state with ID {newGameState.ID}.");
                    if (newGameState.Equals(default)) continue;
                    _viewController.Lock();
                    _viewController.SendNewGameStateToPlayers(newGameState);
                    _logger.Log(LogLevel.Debug, $"Done sending new game state to players in game with ID " +
                                                $"{newGameState.ID}");
                    _viewController.ReleaseLock();
                }
                catch (Exception e)
                {
                    _logger.Log(LogLevel.Error, $"Failed to handle input: {input.ToString()}. Error {e}");
                }
            }
        }
    }

    private GameState HandleInput(Input input)
    {
        // TODO check if input is legal based on the game state once applicable
        _logger.Log(LogLevel.Debug, $"Handling inputs for player with uniqueID {input.Player.UniqueID} and " +
                                                   $"name {input.Player.Name} and input type {input.Type}.");
        switch (input.Type)
        {
            case PlayerInputType.Movement:
                return HandleMovement(input.Player, input.RelatedNode);
            default:
                throw new ArgumentOutOfRangeException();
        }
        _logger.Log(LogLevel.Debug, $"Finished handling inputs for player with ID {input.Player.UniqueID}.");
    }

    private GameState HandleMovement(Player playerCopy, Node toNodeCopy)
    {
        try
        {
            var game = _games.First(state => state.ID == playerCopy.ConnectedGameID);
            var gamePlayer = game.Players.First(player1 => player1.InGameID == playerCopy.InGameID);
            _games.Remove(game);
            game.Players.RemoveAll(player1 => player1.InGameID == playerCopy.InGameID);
            // TODO: Check here if the movement is valid once applicable and dont use toNodeCopy.
            gamePlayer.Position = toNodeCopy;
            game.Players.Add(gamePlayer);
            _games.Add(game);
            _logger.Log(LogLevel.Debug, $"Moved player {playerCopy.InGameID} in {playerCopy.ConnectedGameID} to " +
                                        $"node with nodeID {toNodeCopy.ID}");
            return game;
        }
        catch (InvalidOperationException e)
        {
            _logger.Log(LogLevel.Error, "Failed to move player because the game the player refers to " +
                                        $"doesn't exist or the player isn't in the game. " +
                                        $"GameID: {playerCopy.ConnectedGameID}. InGame PlayerID {playerCopy.InGameID}.");
        }
        catch (Exception e)
        {
            _logger.Log(LogLevel.Error, $"Something went wrong when trying to move the player. Error {e}");
        }

        return default;
    }


    public void Dispose()
    {
        Stop();
        _mainLoopThread.Join();
    }
}