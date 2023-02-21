using System;
using System.Collections.Generic;

namespace SintefDigital_boardGame_server.Core;

public class GameController
{
    private readonly List<GameState> _games;
    private readonly IMultiplayerGameController _viewController;
    private readonly IMultiplayerPlayerInputController _inputController;
    public GameController(IMultiplayerGameController viewController, IMultiplayerPlayerInputController inputController)
    {
        this._games = new List<GameState>();
        this._viewController = viewController;
        this._inputController = inputController;
    }
    
    public void Run()
    {
        while (true)
        {
            var newGames = _inputController.FetchRequestedGameLobbiesWithLobbyNameAndPlayer();
            foreach (var lobbyNameAndPlayer in newGames) HandleNewGameCreation(lobbyNameAndPlayer);

            HandlePlayerInputs();

            break; // TODO: Remove this once the server should actually run forever
        }
    }

    private void HandleNewGameCreation(Tuple<Player, string> lobbyNameAndPlayer)
    {
        var newGame = CreateNewGame(lobbyNameAndPlayer);
        AssignGameToPlayer(lobbyNameAndPlayer.Item1, newGame);
        _games.Add(newGame);
        _viewController.SendNewGameStateToPlayers(newGame);
    }
    
    private GameState CreateNewGame(Tuple<Player, string> lobbyNameAndPlayer)
    {
        var newGame = new GameState
        {
            ID = GenerateUnusedGameID(),
            Name = lobbyNameAndPlayer.Item2,
            Players = new List<Player> { lobbyNameAndPlayer.Item1 }
        };
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
    }

    private void HandlePlayerInputs()
    {
        foreach (var game in _games)
        {
            var playerInputs = _inputController.FetchPlayerInputs(game.ID);
            foreach (var input in playerInputs) HandleInput(input);
        }
    }

    private void HandleInput(Input input)
    {
        // TODO check if input is legal based on the game state once applicable
        switch (input.Type)
        {
            case PlayerInputType.Movement:
                HandleMovement(input.Player, input.ToNode);
                break;
            default:
                throw new ArgumentOutOfRangeException();
        }
    }

    private void HandleMovement(Player player, Node toNode)
    {
        var game = player.ConnectedGame;
        // TODO: Check here if the movement is valid once applicable
        player.Position = toNode;
    }
}