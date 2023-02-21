using SintefDigital_boardGame_server.Core;

namespace SintefDigital_boardGame_server.Test.Core.MockControllers;

public class MockMultiplayerPlayerInput : IMultiplayerPlayerInputController
{
    private int _lastPlayerId = 0;
    private readonly Range _randomNewGameAmountRange;
    private int _howManyTimesToMakeNewGameList;
    private int _timesNewGameListMade = 0;
    private List<Tuple<Player, string>> _wantedNewGames = new List<Tuple<Player, string>>();
    private List<GameState> _createdGameStates = new List<GameState>();
    private List<Tuple<Player, string>> _newGames;

    public MockMultiplayerPlayerInput(Range randomNewGameAmountRange, int howManyTimesToMakeNewGameList)
    {
        _randomNewGameAmountRange = randomNewGameAmountRange;
        _howManyTimesToMakeNewGameList = howManyTimesToMakeNewGameList;
    }
    
    public List<Tuple<Player, string>> FetchRequestedGameLobbiesWithLobbyNameAndPlayer()
    {
        // List<Tuple<Player, string>> newGames = new List<Tuple<Player, string>>();
        //
        // if (AreAllWantedGamesFetched()){
        //     int amountOfNewGames = new Random().Next(_randomNewGameAmountRange.Start.Value, _randomNewGameAmountRange.End.Value);
        //     
        //     Player newPlayer = default;
        //
        //     for (int i = 0; i < amountOfNewGames; i++)
        //     {
        //         newPlayer = CreatePlayerWithIncrementedID();
        //         newGames.Add(new Tuple<Player, string>(newPlayer, newPlayer.Name));
        //     }
        //
        //     if (newPlayer.Equals(default(Player)))
        //     {
        //         newPlayer = CreatePlayerWithIncrementedID();
        //     }
        //
        //     // To check if a new player can be made with same ID but in a different game
        //     newGames.Add(new Tuple<Player, string>(newPlayer, $"{_lastPlayerId + 1}"));
        //
        //     _wantedNewGames.AddRange(newGames);
        //     _timesNewGameListMade++;
        // }
        var clone = new List<Tuple<Player, string>>(_newGames);
        _newGames.Clear();
        return clone;
    }

    public void addNewWantedGames(List<Tuple<Player, string>> wantedGameList)
    {
        _newGames.AddRange(wantedGameList);
    }

    public List<Input> FetchPlayerInputs(int gameID)
    {
        // TODO: Make inputs for the games created
        return new List<Input>();
    }

    public void NotifyNewGameState(GameState state)
    {
        _createdGameStates.Add(state);
    }

    public bool AreAllWantedGamesFetched()
    {
        return _timesNewGameListMade < _howManyTimesToMakeNewGameList;
    }

    public bool AreAllGamesCreated()
    {
        return _createdGameStates.Count == _wantedNewGames.Count;
    }

    private Player CreatePlayerWithIncrementedID()
    {
        return new Player
        {
            ID = ++_lastPlayerId,
            Name = $"{_lastPlayerId}",
            ConnectedGame = default,
            Position = default
        };
    }
    
}