using System;
using System.Collections.Generic;
using SintefDigital_boardGame_server.Core;

namespace SintefDigital_boardGame_server.Communication;

public class InternetMultiplayerController : IMultiplayerViewController, IMultiplayerPlayerInputController
{
    private ReaderWriterLockSlim _lock = new ReaderWriterLockSlim();
    
    public List<Tuple<Player, string>> FetchRequestedGameLobbiesWithLobbyNameAndPlayer()
    {
        VerifyLock();
        var newGames = new List<Tuple<Player, string>>();
        // TODO: Return all the new wanted games
        return newGames;
    }

    public List<Input> FetchPlayerInputs(int gameID)
    {
        VerifyLock();
        var playerInputs = new List<Core.Input>();
        // TODO: Return all the inputs from the game with the given gameID
        return playerInputs;
    }

    public void SendNewGameStateToPlayers(GameState state)
    {
        VerifyLock();
        // TODO: Send game state to the players in the game connected to the game state input
    }

    public void Lock()
    {
        _lock.EnterWriteLock();
    }

    public void ReleaseLock()
    {
        _lock.ExitReadLock();
    }

    public void VerifyLock()
    {
        if (!_lock.IsWriteLockHeld) throw new InvalidOperationException("Before making any calls to this object " +
                                                                        "it needs to be locked unsing Lock() and " +
                                                                        "needs to be released once done!");
    }
}