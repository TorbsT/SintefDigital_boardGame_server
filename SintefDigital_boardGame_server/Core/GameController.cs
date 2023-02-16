using SintefDigital_boardGame_server.Communication;

namespace SintefDigital_boardGame_server.Core;

public class GameController
{
    private List<GameState> games;
    private IMultiplayerGameController viewController;
    private IMultiplayerPlayerInputController inputController;
    public GameController()
    {
        this.games = new List<GameState>();
        var multiplayerController = new InternetMultiplayerController();
        this.viewController = multiplayerController;
        this.inputController = multiplayerController;
    }
    
    public void Run()
    {
        while (true)
        {
            var newGames = inputController.FetchRequestedGameLobbiesWithLobbyNameAndPlayer();
            foreach (var lobbyNameAndPlayer in newGames) HandleNewGameCreation(lobbyNameAndPlayer);

            HandlePlayerInputs();

            break;
        }
    }

    private void HandleNewGameCreation(Tuple<Player, string> lobbyNameAndPlayer)
    {
        var newGame = CreateNewGame(lobbyNameAndPlayer);
        AssignGameToPlayer(lobbyNameAndPlayer.Item1, newGame);
        games.Add(newGame);
        viewController.SendNewGameStateToPlayers(newGame);
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
        foreach (var game in games)
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
        foreach (var game in games)
        {
            var playerInputs = inputController.FetchPlayerInputs(game.ID);
            foreach (var input in playerInputs) HandleInput(input);
        }
    }

    private void HandleInput(Input input)
    {
        //TODO check if input is legal based on the game state once applicable
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
        // TODO: Check here if the movement is valid
        player.Position = toNode;
    }
}