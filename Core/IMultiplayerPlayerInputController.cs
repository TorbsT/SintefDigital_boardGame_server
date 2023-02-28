using System;
using System.Collections.Generic;

namespace Core;

public interface IMultiPlayerInfoPlayerInfoInputController : ILocker
{
    List<(PlayerInfo, string)> FetchRequestedGameLobbiesWithLobbyNameAndPlayerInfo();
    List<Input> FetchPlayerInfoInputs(int gameID);
    int FetchWantedAmountOfUniqueIDs();
}