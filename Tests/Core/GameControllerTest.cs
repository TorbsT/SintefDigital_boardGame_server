using Core;
using Logging;
using System.Xml.Linq;
using Test.Core.MockControllers;
using Xunit;

namespace Test.Core;


public class GameControllerTest : IDisposable
{

    private GameController _gameController;
    private MockMultiPlayerInfoController _mockMultiPlayerInfoController;

    public GameControllerTest()
    {
        (_gameController, _mockMultiPlayerInfoController) = CreateAndRunGameControllerAndMultiPlayerInfoController();
    }

    public void Dispose()
    {
        _gameController.Dispose();
    }

    [Fact]
    public void TestRunningController()
    {
        var mockMultiPlayerInfoController = new MockMultiPlayerInfoController();
        var gameController = new GameController(new ThresholdLogger(LogLevel.Debug, LogLevel.Ignore), mockMultiPlayerInfoController, mockMultiPlayerInfoController);
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
    public void TestCreatingNewGame(int amountOfNewPlayerInfos, int amountOfNewGames)
    {
        Thread.Sleep(10);
        _mockMultiPlayerInfoController.Lock();
        Assert.Empty(_mockMultiPlayerInfoController.FetchCreatedGames());

        List<PlayerInfo> PlayerInfos = MakeRandomPlayerInfoListWithSize(amountOfNewPlayerInfos);

        List<(PlayerInfo, string)> newGames = MakeRandomGameLobbyListWithSize(amountOfNewGames, PlayerInfos);
        _mockMultiPlayerInfoController.AddNewWantedGames(new List<(PlayerInfo, string)>(newGames));
        _mockMultiPlayerInfoController.ReleaseLock();

        List<GameStateInfo> gamesCreatedList = new List<GameStateInfo>();
        for (int _ = 0; _ < 100; _++)
        {
            Thread.Sleep(500);
            _mockMultiPlayerInfoController.Lock();
            gamesCreatedList = _mockMultiPlayerInfoController.FetchCreatedGames();
            _mockMultiPlayerInfoController.ReleaseLock();
            if (gamesCreatedList.Count <= newGames.Count) break;
        }

        Assert.Equal(newGames.Count, gamesCreatedList.Count);

        foreach (var (PlayerInfo, gameName) in newGames)
        {
            Assert.Contains(gamesCreatedList, GameStateInfo =>
            {
                return GameStateInfo.PlayerInfos.Any(gamePlayerInfo => gamePlayerInfo.UniqueID == PlayerInfo.UniqueID) && GameStateInfo.Name == gameName;
            });
        }
    }

    [Fact]
    public void TestPlayerInfoMovement()
    {

        Node startNode = new Node( new NodeInfo() { district = "1", ID = 1});
        

        Node endNode = new Node(new NodeInfo() { district = "1", ID = 1 });

        startNode.AddNeighbour(endNode);
        endNode.AddNeighbour(startNode);

        PlayerInfo PlayerInfo = MakeRandomPlayerInfo();
        PlayerInfo.Position = startNode;
        var newGame = (PlayerInfo, "TestMovement");
        List<(PlayerInfo, string)> newGameList = new List<(PlayerInfo, string)>();
        newGameList.Add(newGame);

        _mockMultiPlayerInfoController.Lock();
        _mockMultiPlayerInfoController.AddNewWantedGames(new List<(PlayerInfo, string)>(newGameList));
        _mockMultiPlayerInfoController.ReleaseLock();

        GameStateInfo GameStateInfo = new GameStateInfo();
        for (int _ = 0; _ < 100; _++)
        {
            Thread.Sleep(100);
            _mockMultiPlayerInfoController.Lock();
            var createdGames = _mockMultiPlayerInfoController.FetchCreatedGames();
            _mockMultiPlayerInfoController.ReleaseLock();
            if (createdGames.Count == 1)
            {
                GameStateInfo = createdGames.First();
                break;
            }
        }

        GameStateInfo.PlayerInfos ??= new List<PlayerInfo>();

        Assert.Contains(GameStateInfo.PlayerInfos, PlayerInfo1 => PlayerInfo1.Position.ID == ((NodeInfo) startNode).ID);

        PlayerInfo = GameStateInfo.PlayerInfos.First(PlayerInfo1 => PlayerInfo1.UniqueID == PlayerInfo.UniqueID);

        Input input = new Input();
        input.PlayerInfo = PlayerInfo;
        input.Type = PlayerInfoInputType.Movement;
        input.RelatedNode = endNode;

        _mockMultiPlayerInfoController.Lock();
        _mockMultiPlayerInfoController.AddInput(input);
        _mockMultiPlayerInfoController.ReleaseLock();

        Thread.Sleep(500); // Let the game controller handle the inputs

        _mockMultiPlayerInfoController.Lock();
        GameStateInfo = _mockMultiPlayerInfoController.FetchCreatedGames().First();
        _mockMultiPlayerInfoController.ReleaseLock();

        Assert.Contains(GameStateInfo.PlayerInfos, PlayerInfo1 => PlayerInfo1.Position.ID == ((NodeInfo) endNode).ID);
    }

    private (GameController, MockMultiPlayerInfoController) CreateAndRunGameControllerAndMultiPlayerInfoController()
    {
        var mockMultiPlayerInfoController = new MockMultiPlayerInfoController();
        var gameController = new GameController(new ThresholdLogger(LogLevel.Debug, LogLevel.Ignore), mockMultiPlayerInfoController, mockMultiPlayerInfoController);
        gameController.Run();
        return (gameController, mockMultiPlayerInfoController);
    }

    private List<(PlayerInfo, string)> MakeRandomGameLobbyListWithSize(int listSize, List<PlayerInfo> PlayerInfos)
    {
        List<(PlayerInfo, string)> newGames = new List<(PlayerInfo, string)>(listSize);
        int PlayerInfoIndex = 0;
        for (int _ = 0; _ < listSize; _++)
        {
            PlayerInfo PlayerInfo = default;
            if (PlayerInfoIndex == PlayerInfos.Count) PlayerInfoIndex = 0;
            if (PlayerInfos.Count != 0) PlayerInfo = PlayerInfos[PlayerInfoIndex];
            newGames.Add(MakeRandomLobbyWithPlayerInfo(PlayerInfo));
        }

        return newGames;
    }

    private (PlayerInfo, string) MakeRandomLobbyWithPlayerInfo(PlayerInfo PlayerInfo)
    {
        Random generator = new Random();
        return (PlayerInfo, generator.Next().ToString());
    }

    private List<PlayerInfo> MakeRandomPlayerInfoListWithSize(int listSize)
    {
        List<PlayerInfo> PlayerInfos = new List<PlayerInfo>(listSize);
        for (int _ = 0; _ < listSize; _++)
        {
            PlayerInfo newPlayerInfo = MakeRandomPlayerInfo();
            while (PlayerInfos.Any(PlayerInfo => PlayerInfo.UniqueID == newPlayerInfo.UniqueID))
            {
                newPlayerInfo = MakeRandomPlayerInfo();
            }
            PlayerInfos.Add(newPlayerInfo);
        }

        return PlayerInfos;
    }

    private PlayerInfo MakeRandomPlayerInfo()
    {

        PlayerInfo PlayerInfo = new PlayerInfo();
        PlayerInfo.Name = MakeRandomNumber().ToString();
        PlayerInfo.UniqueID = MakeRandomNumber();
        PlayerInfo.InGameID = 1;
        return PlayerInfo;
    }

    private int MakeRandomNumber()
    {
        Random generator = new Random();
        return generator.Next();
    }

    private NodeInfo MakeRandomNode()
    {
        NodeInfo node = default;
        node.ID = MakeRandomNumber();
        return node;
    }
}