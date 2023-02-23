using SintefDigital_boardGame_server.Core;
using SintefDigital_boardGame_server.Logging;
using SintefDigital_boardGame_server.Test.Core.MockControllers;
using Xunit;

namespace SintefDigital_boardGame_server.Test.Core;


public class GameControllerTest : IDisposable
{

    private GameController _gameController;
    private MockMultiplayerController _mockMultiplayerController;

    public GameControllerTest()
    {
        (_gameController, _mockMultiplayerController) = CreateAndRunGameControllerAndMultiplayerController();
    }

    public void Dispose()
    {
        _gameController.Dispose();
    }
    
    [Fact]
    public void TestRunningController()
    {
        var mockMultiPlayerController = new MockMultiplayerController();
        var gameController = new GameController(new ThresholdLogger(LogLevel.Debug, LogLevel.Ignore), mockMultiPlayerController, mockMultiPlayerController);
        Assert.False(gameController.IsMainLoopRunning());
        gameController.Run();
        Assert.True(gameController.IsMainLoopRunning());
        gameController.Dispose();
    }
    
    [Theory]
    [InlineData(0, 0)]
    [InlineData(0, 1)]
    [InlineData(1, 1)]
    [InlineData(5, 10)]
    [InlineData(100, 110)]
    [InlineData(1000, 1000)]
    public void TestCreatingNewGame(int amountOfNewPlayers, int amountOfNewGames)
    {
        Thread.Sleep(10);
        _mockMultiplayerController.Lock();
        Assert.Empty(_mockMultiplayerController.FetchCreatedGames());

        List<Player> players = MakeRandomPlayerListWithSize(amountOfNewPlayers);
        
        List<(Player,string)> newGames = MakeRandomGameLobbyListWithSize(amountOfNewGames, players);
        _mockMultiplayerController.AddNewWantedGames(new List<(Player,string)>(newGames));
        _mockMultiplayerController.ReleaseLock();
        
        List<GameState> gamesCreatedList = new List<GameState>();
        for (int _ = 0; _ < 100; _++)
        {
            Thread.Sleep(500);
            _mockMultiplayerController.Lock();
            gamesCreatedList = _mockMultiplayerController.FetchCreatedGames();
            _mockMultiplayerController.ReleaseLock();
            if (gamesCreatedList.Count <= newGames.Count) break;
        }
        
        Assert.Equal(newGames.Count, gamesCreatedList.Count);

        foreach (var (player, gameName) in newGames)
        {
            Assert.Contains(gamesCreatedList, gameState =>
            {
                return gameState.Players.Any(gamePlayer => gamePlayer.UniqueID == player.UniqueID) && gameState.Name == gameName;
            });
        }
    }

    [Fact]
    public void TestPlayerMovement()
    {
        
        Node startNode = new Node();
        startNode.ID = 1;
        startNode.Neighbours = new List<Node>();
        
        Node endNode = new Node();
        endNode.ID = 2;
        endNode.Neighbours = new List<Node>();
        
        startNode.Neighbours.Add(endNode);
        endNode.Neighbours.Add(startNode);
        
        Player player = MakeRandomPlayer();
        player.Position = startNode;
        var newGame = (player, "TestMovement");
        List<(Player, string)> newGameList = new List<(Player, string)>();
        newGameList.Add(newGame);
        
        _mockMultiplayerController.Lock();
        _mockMultiplayerController.AddNewWantedGames(new List<(Player, string)>(newGameList));
        _mockMultiplayerController.ReleaseLock();

        GameState gameState = new GameState();
        for (int _ = 0; _ < 100; _++)
        {
            Thread.Sleep(100);
            _mockMultiplayerController.Lock();
            var createdGames = _mockMultiplayerController.FetchCreatedGames();
            _mockMultiplayerController.ReleaseLock();
            if (createdGames.Count == 1)
            {
                gameState = createdGames.First();
                break;
            }
        }

        gameState.Players ??= new List<Player>();

        Assert.Contains(gameState.Players, player1 => player1.Position.ID == startNode.ID);

        player = gameState.Players.First(player1 => player1.UniqueID == player.UniqueID);
        
        Input input = new Input();
        input.Player = player;
        input.Type = PlayerInputType.Movement;
        input.RelatedNode = endNode;
        
        _mockMultiplayerController.Lock();
        _mockMultiplayerController.AddInput(input);
        _mockMultiplayerController.ReleaseLock();
        
        Thread.Sleep(500); // Let the game controller handle the inputs
        
        _mockMultiplayerController.Lock();
        gameState = _mockMultiplayerController.FetchCreatedGames().First();
        _mockMultiplayerController.ReleaseLock();
        
        Assert.Contains(gameState.Players, player1 => player1.Position.ID == endNode.ID);
    }

    private (GameController, MockMultiplayerController) CreateAndRunGameControllerAndMultiplayerController()
    {
        var mockMultiPlayerController = new MockMultiplayerController();
        var gameController = new GameController(new ThresholdLogger(LogLevel.Debug, LogLevel.Ignore), mockMultiPlayerController, mockMultiPlayerController);
        gameController.Run();
        return (gameController, mockMultiPlayerController);
    }
    
    private List<(Player,string)> MakeRandomGameLobbyListWithSize(int listSize, List<Player> players)
    {
        List<(Player,string)> newGames = new List<(Player,string)>(listSize);
        int playerIndex = 0;
        for (int _ = 0; _ < listSize; _++)
        {
            Player player = default;
            if (playerIndex == players.Count) playerIndex = 0;
            if (players.Count != 0) player = players[playerIndex];
            newGames.Add(MakeRandomLobbyWithPlayer(player));
        }

        return newGames;
    }

    private (Player,string) MakeRandomLobbyWithPlayer(Player player)
    {
        Random generator = new Random();
        return (player, generator.Next().ToString());
    }

    private List<Player> MakeRandomPlayerListWithSize(int listSize)
    {
        List<Player> players = new List<Player>(listSize);
        for (int _ = 0; _ < listSize; _++)
        {
            Player newPlayer = MakeRandomPlayer();
            while (players.Any(player => player.UniqueID == newPlayer.UniqueID))
            {
                newPlayer = MakeRandomPlayer();
            }
            players.Add(newPlayer);
        }

        return players;
    }

    private Player MakeRandomPlayer()
    {
        
        Player player = new Player();
        player.Name = MakeRandomNumber().ToString();
        player.UniqueID = MakeRandomNumber();
        player.InGameID = 1;
        return player;
    }

    private int MakeRandomNumber()
    {
        Random generator = new Random();
        return generator.Next();
    }
    
    private Node MakeRandomNode()
    {
        Node node = default;
        node.ID = MakeRandomNumber();
        return node;
    }
}