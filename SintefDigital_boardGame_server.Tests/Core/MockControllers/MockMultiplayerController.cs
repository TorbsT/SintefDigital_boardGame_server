using SintefDigital_boardGame_server.Core;

namespace SintefDigital_boardGame_server.Test.Core.MockControllers;

public class MockMultiplayerController : IMultiplayerViewController, IMultiplayerPlayerInputController
{
    private List<GameState> _createdGameStates = new List<GameState>();
    private List<Tuple<Player, string>> _newGames = new List<Tuple<Player, string>>();
    private ReaderWriterLockSlim _lock = new ReaderWriterLockSlim();

    public MockMultiplayerController()
    {
        
    }
    
    public void SendNewGameStateToPlayers(GameState state)
    {
        VerifyLock();
        _createdGameStates.Add(state);
    }

    public List<Tuple<Player, string>> FetchRequestedGameLobbiesWithLobbyNameAndPlayer()
    {
        VerifyLock();
        var clone = new List<Tuple<Player, string>>(_newGames);
        _newGames.Clear();
        return clone;
    }

    public void addNewWantedGames(List<Tuple<Player, string>> wantedGameList)
    {
        VerifyLock();
        _newGames.AddRange(wantedGameList);
    }

    public List<Input> FetchPlayerInputs(int gameID)
    {
        VerifyLock();
        // TODO: Make inputs for the games created
        return new List<Input>();
    }

    public List<GameState> FetchCreatedGames()
    {
        VerifyLock();
        return new List<GameState>(_createdGameStates);
    }

    public void Lock()
    {
        _lock.EnterWriteLock();
    }

    public void ReleaseLock()
    {
        _lock.ExitWriteLock();
    }
    
    public void VerifyLock()
    {
        if (!_lock.IsWriteLockHeld) throw new InvalidOperationException("Before making any calls to this object " +
                                                                        "it needs to be locked unsing Lock() and " +
                                                                        "needs to be released once done!");
    }
}