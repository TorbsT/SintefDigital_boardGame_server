using Core;
using Logging;
using System.ComponentModel;
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
    [InlineData(0)]
    [InlineData(1)]
    [InlineData(5)]
    [InlineData(50)]
    [InlineData(500)]
    [InlineData(5000)]
    [InlineData(50000)]
    public void TestUniquePlayerIDs(int amountOfPlayersToCreate)
    {
        _mockMultiPlayerInfoController.Lock();
        for (int _ = 0; _ < amountOfPlayersToCreate; _++) _mockMultiPlayerInfoController.NotifyWantID();
        _mockMultiPlayerInfoController.ReleaseLock();
        
        List<int> IDs = new List<int>();
        int msTimeForEachUniqueID = 50;
        int averageMaxRetryForEachID = 10;
        for (int _ = 0; _ < amountOfPlayersToCreate*averageMaxRetryForEachID; _++)
        {
            _mockMultiPlayerInfoController.Lock();
            var (gotID, newID) = _mockMultiPlayerInfoController.FetchUniqueID();
            _mockMultiPlayerInfoController.ReleaseLock();
            if (gotID)
            {
                Assert.True(!IDs.Contains(newID));
                IDs.Add(newID);
            } else Thread.Sleep(msTimeForEachUniqueID);
            if (IDs.Count >= amountOfPlayersToCreate) break;
        }
        Assert.Equal(IDs.Count, amountOfPlayersToCreate);
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
        _mockMultiPlayerInfoController.Lock();
        Assert.Empty(_mockMultiPlayerInfoController.FetchCreatedGames());
        _mockMultiPlayerInfoController.ReleaseLock();
        
        List<PlayerInfo> playerInfos = MakeRandomPlayerInfoListWithSize(amountOfNewPlayerInfos);

        List<(PlayerInfo, string)> newGames = MakeRandomGameLobbyListWithSize(amountOfNewGames, playerInfos);
        _mockMultiPlayerInfoController.Lock();
        _mockMultiPlayerInfoController.AddNewWantedGames(new List<(PlayerInfo, string)>(newGames));
        _mockMultiPlayerInfoController.ReleaseLock();

        List<GameStateInfo> gamesCreatedList = new List<GameStateInfo>();
        int timesToCheckForNewGame = 10 * amountOfNewPlayerInfos;
        int msToSleepBetweenEachCheck = 1;
        for (int _ = 0; _ < timesToCheckForNewGame; _++)
        {
            Thread.Sleep(msToSleepBetweenEachCheck);
            _mockMultiPlayerInfoController.Lock();
            gamesCreatedList = _mockMultiPlayerInfoController.FetchCreatedGames();
            _mockMultiPlayerInfoController.ReleaseLock();
            if (gamesCreatedList.Count >= newGames.Count) break;
        }
        
        // Because a player cannot be in more than one game at a time
        Assert.Equal(amountOfNewPlayerInfos, gamesCreatedList.Count);
        
        var actualGamesToCreateFromFullList = new List<(PlayerInfo, string)>();
        for (int i = 0; i < newGames.Count; i++)
        {
            if (actualGamesToCreateFromFullList.Any((tuple) => newGames[i].Item1.UniqueID == tuple.Item1.UniqueID)) continue;
            actualGamesToCreateFromFullList.Add(newGames[i]);
        }

        foreach (var (PlayerInfo, gameName) in actualGamesToCreateFromFullList)
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

        Node startNode = new Node( new NodeInfo() {Name = "Start", ID = 1});
        

        Node endNode = new Node(new NodeInfo() {Name = "End", ID = 2 });

        startNode.AddNeighbour(endNode);
        endNode.AddNeighbour(startNode);

        PlayerInfo playerInfo = MakeRandomPlayerInfo();
        playerInfo.Position = startNode;
        var newGame = (playerInfo, "TestMovement");
        List<(PlayerInfo, string)> newGameList = new List<(PlayerInfo, string)>();
        newGameList.Add(newGame);

        _mockMultiPlayerInfoController.Lock();
        _mockMultiPlayerInfoController.AddNewWantedGames(new List<(PlayerInfo, string)>(newGameList));
        _mockMultiPlayerInfoController.ReleaseLock();

        GameStateInfo gameStateInfo = new GameStateInfo();
        for (int _ = 0; _ < 100; _++)
        {
            Thread.Sleep(100);
            _mockMultiPlayerInfoController.Lock();
            var createdGames = _mockMultiPlayerInfoController.FetchCreatedGames();
            _mockMultiPlayerInfoController.ReleaseLock();
            if (createdGames.Count == 1)
            {
                gameStateInfo = createdGames.First();
                break;
            }
        }

        gameStateInfo.PlayerInfos ??= new List<PlayerInfo>();

        Assert.Contains(gameStateInfo.PlayerInfos, playerInfo1 => playerInfo1.Position.ID == ((NodeInfo) startNode).ID);

        playerInfo = gameStateInfo.PlayerInfos.First(playerInfo1 => playerInfo1.UniqueID == playerInfo.UniqueID);

        Input input = new Input();
        input.PlayerInfo = playerInfo;
        input.Type = PlayerInfoInputType.Movement;
        input.RelatedNode = endNode;

        _mockMultiPlayerInfoController.Lock();
        _mockMultiPlayerInfoController.AddInput(input);
        _mockMultiPlayerInfoController.ReleaseLock();

        Thread.Sleep(500); // Let the game controller handle the inputs

        _mockMultiPlayerInfoController.Lock();
        gameStateInfo = _mockMultiPlayerInfoController.FetchCreatedGames().First();
        _mockMultiPlayerInfoController.ReleaseLock();

        Assert.Contains(gameStateInfo.PlayerInfos, player => player.UniqueID == playerInfo.UniqueID);

        Assert.Contains(gameStateInfo.PlayerInfos, playerInfo1 => playerInfo1.Position.ID == ((NodeInfo) endNode).ID);
    }

    private (GameController, MockMultiPlayerInfoController) CreateAndRunGameControllerAndMultiPlayerInfoController()
    {
        var mockMultiPlayerInfoController = new MockMultiPlayerInfoController();
        var gameController = new GameController(new ThresholdLogger(LogLevel.Debug, LogLevel.Ignore), mockMultiPlayerInfoController, mockMultiPlayerInfoController);
        gameController.Run();
        return (gameController, mockMultiPlayerInfoController);
    }

    private List<(PlayerInfo, string)> MakeRandomGameLobbyListWithSize(int listSize, List<PlayerInfo> playerInfos)
    {
        List<(PlayerInfo, string)> newGames = new List<(PlayerInfo, string)>(listSize);
        int playerInfoIndex = 0;
        for (int _ = 0; _ < listSize; _++)
        {
            if (playerInfos.Count == 0) break;
            PlayerInfo playerInfo = playerInfos[playerInfoIndex++];
            if (playerInfoIndex == playerInfos.Count) playerInfoIndex = 0;
            newGames.Add(MakeRandomLobbyWithPlayerInfo(playerInfo));
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
        PlayerInfo.UniqueID = RetrieveUniquePlayerID();
        PlayerInfo.InGameID = 1;
        return PlayerInfo;
    }

    private int MakeRandomNumber()
    {
        Random generator = new Random();
        return generator.Next();
    }

    private int RetrieveUniquePlayerID()
    {
        _mockMultiPlayerInfoController.Lock();
        _mockMultiPlayerInfoController.NotifyWantID();
        _mockMultiPlayerInfoController.ReleaseLock();
        int id = 0;
        bool gotID = false;
        for (int _ = 0; _ < 10_000; _++)
        {
            Thread.Sleep(10);
            _mockMultiPlayerInfoController.Lock();
            var (isNewID, uniqueID) = _mockMultiPlayerInfoController.FetchUniqueID();
            _mockMultiPlayerInfoController.ReleaseLock();
            if (isNewID)
            {
                id = uniqueID;
                gotID = true;
                break;
            }
        }
        if (!gotID) throw new Exception("Failed to get unique ID!");
        return id;
    }

    private NodeInfo MakeRandomNode()
    {
        NodeInfo node = default;
        node.ID = MakeRandomNumber();
        return node;
    }
}