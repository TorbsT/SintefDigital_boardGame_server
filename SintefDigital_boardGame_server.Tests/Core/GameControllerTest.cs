using SintefDigital_boardGame_server.Core;
using SintefDigital_boardGame_server.Logging;
using SintefDigital_boardGame_server.Test.Core.MockControllers;
using Xunit;

namespace SintefDigital_boardGame_server.Test.Core;


public class GameControllerTest
{

    
    
    [Fact]
    public void TestRunningController()
    {
        var mockMultiPlayerController = new MockMultiplayerController();
        var gameController = new GameController(new ThresholdLogger(LogLevel.Ignore, LogLevel.Ignore), mockMultiPlayerController, mockMultiPlayerController);
        Assert.False(gameController.IsMainLoopRunning());
        gameController.Run();
        Assert.True(gameController.IsMainLoopRunning());
        
    }
    
    [Theory]
    [InlineData(0, 0)]
    [InlineData(0, 1)]
    [InlineData(1, 1)]
    [InlineData(5, 10)]
    [InlineData(100, 110)]
    [InlineData(100000, 100000)]
    public void TestCreatingNewGame(int amountOfNewPlayers, int amountOfNewGames)
    {
        var mockMultiPlayerController = new MockMultiplayerController();
        var gameController = new GameController(new ThresholdLogger(LogLevel.Ignore, LogLevel.Ignore), mockMultiPlayerController, mockMultiPlayerController);
        gameController.Run();
        
        // TODO: Write the tests!
        Thread.Sleep(10);
        mockMultiPlayerController.Lock();
        Assert.Empty(mockMultiPlayerController.FetchCreatedGames());

        List<Player> players = MakeRandomPlayerListWithSize(amountOfNewPlayers);
        
        List<Tuple<Player, string>> newGames = MakeRandomGameLobbyListWithSize(amountOfNewGames, players);
        mockMultiPlayerController.AddNewWantedGames(new List<Tuple<Player, string>>(newGames));
        mockMultiPlayerController.ReleaseLock();

        int counter = 0;
        List<GameState> gamesCreatedList = new List<GameState>();
        while (gamesCreatedList.Count < newGames.Count || counter >= 100)
        {
            Thread.Sleep(100);
            mockMultiPlayerController.Lock();
            gamesCreatedList = mockMultiPlayerController.FetchCreatedGames();
            mockMultiPlayerController.ReleaseLock();
            counter++;
        }
        Assert.Equal(newGames.Count, gamesCreatedList.Count);
    }

    private List<Tuple<Player,string>> MakeRandomGameLobbyListWithSize(int listSize, List<Player> players)
    {
        List<Tuple<Player, string>> newGames = new List<Tuple<Player, string>>(listSize);
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

    private Tuple<Player, string> MakeRandomLobbyWithPlayer(Player player)
    {
        Random generator = new Random();
        return new Tuple<Player, string>(player, generator.Next().ToString());
    }

    private List<Player> MakeRandomPlayerListWithSize(int listSize)
    {
        List<Player> players = new List<Player>(listSize);
        for (int _ = 0; _ < listSize; _++)
        {
            Player newPlayer = MakeRandomPlayer();
            while (players.Any(player => player.uniqueID == newPlayer.uniqueID))
            {
                newPlayer = MakeRandomPlayer();
            }
            players.Add(newPlayer);
        }

        return players;
    }

    private Player MakeRandomPlayer()
    {
        Random generator = new Random();
        Player player = new Player();
        player.Name = generator.Next().ToString();
        player.uniqueID = generator.Next();
        player.inGameID = 1;
        return player;
    }
}