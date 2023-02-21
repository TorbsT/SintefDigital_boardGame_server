using SintefDigital_boardGame_server.Core;

namespace SintefDigital_boardGame_server.Test.Core.MockControllers;

public class MockMultiplayerController : IMultiplayerViewController, IMultiplayerPlayerInputController
{
    private List<GameState> _createdGameStates = new List<GameState>();
    private List<Tuple<Player, string>> _newGames = new List<Tuple<Player, string>>();

    public MockMultiplayerController()
    {
        
    }
    
    public void SendNewGameStateToPlayers(GameState state)
    {
        _createdGameStates.Add(state);
    }

    public List<Tuple<Player, string>> FetchRequestedGameLobbiesWithLobbyNameAndPlayer()
    {
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

    public List<GameState> FetchCreatedGames()
    {
        return new List<GameState>(_createdGameStates);
    }
}