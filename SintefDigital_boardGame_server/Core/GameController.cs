using System;
using System.Collections.Generic;
using System.Threading;

using SintefDigital_boardGame_server.Logging;


namespace SintefDigital_boardGame_server.Core;

public class GameController
{
    private readonly List<GameState> _games;
    private readonly RwLock<IMultiplayerViewController> _viewController;
    private readonly RwLock<IMultiplayerPlayerInputController> _inputController;
    private readonly ILogger _logger;
    private Thread _mainLoopThread;
    private RwLock<bool> _stopMainThread = new RwLock<bool>(false);

    public GameController(ILogger logger, RwLock<IMultiplayerViewController> viewController, RwLock<IMultiplayerPlayerInputController> inputController)
    {
        this._games = new List<GameState>();
        this._viewController = viewController;
        this._inputController = inputController;
        this._logger = logger;
    }
    
    //TODO: Make an instantiator that gives a GameController that is threadsafe.

    public void Run()
    {
        if (_mainLoopThread != null)
        {
            _logger.Log(LogLevel.Error, "The GameController is already running!");
            return;
        }

        _mainLoopThread = new Thread(RunMainLoop);
        _mainLoopThread.Start();
    }

    public void Stop()
    {
        var stopThread = _stopMainThread.Lock();
        
    }

    private void RunMainLoop()
    {
        bool stop = _stopMainThread.Lock();
        _stopMainThread.ReleaseLock();
        
        while (!stop){
            _logger.Log(LogLevel.Debug, "Getting the new game requests.");
            try
            {
                var inputController = _inputController.Lock();
                List<Tuple<Player, string>>
                    newGames = inputController.FetchRequestedGameLobbiesWithLobbyNameAndPlayer();
                foreach (var lobbyNameAndPlayer in newGames) HandleNewGameCreation(lobbyNameAndPlayer);
            }
            catch (Exception e)
            {
                _logger.Log(LogLevel.Error, $"Failed to get and create new game(s). Error {e}.");
            }
            finally
            {
                _inputController.ReleaseLock();
            }


            _logger.Log(LogLevel.Debug, "Done getting the new game requests.");

            _logger.Log(LogLevel.Debug, "Getting player inputs and handling them.");
            try
            {
                HandlePlayerInputs();
            }
            catch (Exception e)
            {
                _logger.Log(LogLevel.Error, $"Failed to handle player inputs {e}.");
            }
            _logger.Log(LogLevel.Debug, "Done handling player inputs.");
            
            stop = _stopMainThread.Lock();
            _stopMainThread.ReleaseLock();
        }
    }

    private void HandleNewGameCreation(Tuple<Player, string> lobbyNameAndPlayer)
    {
        var newGame = CreateNewGame(lobbyNameAndPlayer);
        AssignGameToPlayer(lobbyNameAndPlayer.Item1, newGame);
        _games.Add(newGame);
        try
        {
            var viewController = _viewController.Lock();
            viewController.SendNewGameStateToPlayers(newGame);
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
    
    private GameState CreateNewGame(Tuple<Player, string> lobbyNameAndPlayer)
    {
        _logger.Log(LogLevel.Debug, "Creating new game state.");
        var newGame = new GameState
        {
            ID = GenerateUnusedGameID(),
            Name = lobbyNameAndPlayer.Item2,
            Players = new List<Player> { lobbyNameAndPlayer.Item1 }
        };
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

    private void AssignGameToPlayer(Player player, GameState game)
    {
        player.ConnectedGame = game;
        _logger.Log(LogLevel.Debug, $"Assigned player with ID {player.ID} to game with id {game.ID}");
    }

    private async void HandlePlayerInputs()
    {
        foreach (var game in _games)
        {
            List<Input> playerInputs = new List<Input>();
            try
            {
                var inputController = _inputController.Lock();
                playerInputs = inputController.FetchPlayerInputs(game.ID);
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
            
            foreach (var input in playerInputs) HandleInput(input);
        }
    }

    private void HandleInput(Input input)
    {
        // TODO check if input is legal based on the game state once applicable
        _logger.Log(LogLevel.Debug, $"Handling inputs for player with ID {input.Player.ID} and " +
                                                   $"name {input.Player.Name} and input type {input.Type}.");
        switch (input.Type)
        {
            case PlayerInputType.Movement:
                HandleMovement(input.Player, input.ToNode);
                break;
            default:
                throw new ArgumentOutOfRangeException();
        }
        _logger.Log(LogLevel.Debug, $"Finished handling inputs for player with ID {input.Player.ID}");
    }

    private void HandleMovement(Player player, Node toNode)
    {
        var game = player.ConnectedGame;
        // TODO: Check here if the movement is valid once applicable
        player.Position = toNode;
    }
}