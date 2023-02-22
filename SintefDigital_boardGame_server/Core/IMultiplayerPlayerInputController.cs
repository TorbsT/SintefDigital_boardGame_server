using System;
using System.Collections.Generic;

namespace SintefDigital_boardGame_server.Core;

public interface IMultiplayerPlayerInputController : ILocker
{
    List<Tuple<Player, string>> FetchRequestedGameLobbiesWithLobbyNameAndPlayer();
    List<Input> FetchPlayerInputs(int gameID);
}