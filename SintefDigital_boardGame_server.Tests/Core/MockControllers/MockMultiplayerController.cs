using SintefDigital_boardGame_server.Core;

namespace SintefDigital_boardGame_server.Test.Core.MockControllers;

public class MockMultiplayerController : IMultiplayerViewController, IMultiplayerPlayerInputController
{
    private List<GameState> _createdGameStates = new List<GameState>();
    private List<(Player,string)> _newGames = new List<(Player,string)>();
    private ReaderWriterLockSlim _lock = new ReaderWriterLockSlim();
    private List<Input> _inputs = new List<Input>();

    public MockMultiplayerController()
    {
        
    }
    
    public void SendNewGameStateToPlayers(GameState state)
    {
        VerifyLock();
        Console.WriteLine(_createdGameStates.Count);
        _createdGameStates.RemoveAll(gameState => gameState.ID == state.ID);
        Console.WriteLine(_createdGameStates.Count);
        _createdGameStates.Add(state);
    }

    public List<(Player,string)> FetchRequestedGameLobbiesWithLobbyNameAndPlayer()
    {
        VerifyLock();
        var clone = new List<(Player,string)>(_newGames);
        _newGames.Clear();
        return clone;
    }

    public void AddNewWantedGames(List<(Player,string)> wantedGameList)
    {
        VerifyLock();
        _newGames.AddRange(wantedGameList);
    }

    public List<Input> FetchPlayerInputs(int gameID)
    {
        VerifyLock();
        List<Input> inputs = new List<Input>(_inputs);
        _inputs.Clear();
        return inputs;
    }

    public List<GameState> FetchCreatedGames()
    {
        VerifyLock();
        return new List<GameState>(_createdGameStates);
    }

    public void AddInput(Input input)
    {
        VerifyLock();
        _inputs.Add(input);
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