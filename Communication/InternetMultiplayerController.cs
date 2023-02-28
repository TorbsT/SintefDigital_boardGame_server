using Core;
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using Microsoft.AspNetCore.Mvc;
using Newtonsoft.Json;

namespace Communication;

public class InternetMultiPlayerInfoController : IMultiPlayerInfoViewController, IMultiPlayerInfoPlayerInfoInputController
{
    private ReaderWriterLockSlim _lock = new ReaderWriterLockSlim();

    public List<(PlayerInfo, string)> FetchRequestedGameLobbiesWithLobbyNameAndPlayerInfo()
    {
        VerifyLock();
        var newGames = new List<(PlayerInfo, string)>();
        // TODO: Return all the new wanted games
        return newGames;
    }

    public List<Input> FetchPlayerInfoInputs(int gameID)
    {
        VerifyLock();
        var PlayerInfoInputs = new List<Input>();
        // TODO: Return all the inputs from the game with the given gameID
        return PlayerInfoInputs;
    }

    public void SendNewGameStateInfoToPlayerInfos(GameStateInfo state)
    {
        VerifyLock();
        // TODO: Send game state to the PlayerInfos in the game connected to the game state input
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

    public void HandleGeneratedUniqueIDs(List<int> uniqueIDs)
    {
        throw new NotImplementedException();
    }

    public int FetchWantedAmountOfUniqueIDs()
    {
        throw new NotImplementedException();
    }
}