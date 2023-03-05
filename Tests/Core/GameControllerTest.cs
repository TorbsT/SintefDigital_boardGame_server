using Core;
using Logging;
using System.ComponentModel;
using System.Xml.Linq;
using Xunit;

namespace Test.Core;

public class GameControllerTest
{

    private GameController _gameController;

    public GameControllerTest(GameController controller)
    {
        _gameController = controller;
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
        
        List<int> IDs = new List<int>();
        for (int _ = 0; _ < amountOfPlayersToCreate; _++)
        {
            int newID = _gameController.MakeNewPlayerID();
            Assert.True(!IDs.Contains(newID));
            IDs.Add(newID);
            if (IDs.Count >= amountOfPlayersToCreate) break;
        }
        Assert.Equal(IDs.Count, amountOfPlayersToCreate);
    }

    [Theory]
    [InlineData(0, 0)]
    [InlineData(0, 1)]
    [InlineData(1, 1)]
    [InlineData(3, 3)]
    [InlineData(5, 10)]
    [InlineData(100, 110)]
    [InlineData(1000, 1000)]
    public void TestCreatingNewGame(int amountOfNewPlayerInfos, int amountOfNewGames)
    {
        List<PlayerInfo> playerInfos = MakeRandomPlayerInfoListWithSize(amountOfNewPlayerInfos);

        List<(PlayerInfo, string)> newGames = MakeRandomGameLobbyListWithSize(amountOfNewGames, playerInfos);
        foreach (var newGame in newGames)
        {
            try
            {
                _gameController.CreateNewGame(newGame);
            }
            catch (Exception)
            {
                
            }
        }

        List<GameStateInfo> gamesCreatedList = _gameController.GetGameStateInfos();

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
        
        GameStateInfo gameStateInfo = _gameController.CreateNewGame(newGame);

        gameStateInfo.PlayerInfos ??= new List<PlayerInfo>();

        Assert.Contains(gameStateInfo.PlayerInfos, playerInfo1 => playerInfo1.Position.ID == ((NodeInfo) startNode).ID);

        playerInfo = gameStateInfo.PlayerInfos.First(playerInfo1 => playerInfo1.UniqueID == playerInfo.UniqueID);

        Input input = new Input();
        input.PlayerInfo = playerInfo;
        input.Type = PlayerInfoInputType.Movement;
        input.RelatedNode = endNode;

        _gameController.HandlePlayerInput(input);

        Thread.Sleep(500); // Let the game controller handle the inputs

        gameStateInfo = _gameController.GetGameStateInfos().First();

        Assert.Contains(gameStateInfo.PlayerInfos, player => player.UniqueID == playerInfo.UniqueID);

        Assert.Contains(gameStateInfo.PlayerInfos, playerInfo1 => playerInfo1.Position.ID == ((NodeInfo) endNode).ID);
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
        PlayerInfo.UniqueID = _gameController.MakeNewPlayerID();
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