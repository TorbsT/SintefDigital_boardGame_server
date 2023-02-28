using Core;

namespace Test.Core.MockControllers;

public class MockMultiPlayerInfoController : IMultiPlayerInfoViewController, IMultiPlayerInfoPlayerInfoInputController
{
    private List<GameStateInfo> _createdGameStateInfos = new List<GameStateInfo>();
    private List<(PlayerInfo, string)> _newGames = new List<(PlayerInfo, string)>();
    private ReaderWriterLockSlim _lock = new ReaderWriterLockSlim();
    private List<Input> _inputs = new List<Input>();

    public MockMultiPlayerInfoController()
    {

    }

    public void SendNewGameStateInfoToPlayerInfos(GameStateInfo state)
    {
        VerifyLock();
        Console.WriteLine(_createdGameStateInfos.Count);
        _createdGameStateInfos.RemoveAll(GameStateInfo => GameStateInfo.ID == state.ID);
        Console.WriteLine(_createdGameStateInfos.Count);
        _createdGameStateInfos.Add(state);
    }

    public List<(PlayerInfo, string)> FetchRequestedGameLobbiesWithLobbyNameAndPlayerInfo()
    {
        VerifyLock();
        var clone = new List<(PlayerInfo, string)>(_newGames);
        _newGames.Clear();
        return clone;
    }

    public void AddNewWantedGames(List<(PlayerInfo, string)> wantedGameList)
    {
        VerifyLock();
        _newGames.AddRange(wantedGameList);
    }

    public List<Input> FetchPlayerInfoInputs(int gameID)
    {
        VerifyLock();
        List<Input> inputs = new List<Input>(_inputs);
        _inputs.Clear();
        return inputs;
    }

    public List<GameStateInfo> FetchCreatedGames()
    {
        VerifyLock();
        return new List<GameStateInfo>(_createdGameStateInfos);
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