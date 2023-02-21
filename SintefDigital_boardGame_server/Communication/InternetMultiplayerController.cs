using System;
using System.Collections.Generic;
using SintefDigital_boardGame_server.Core;

namespace SintefDigital_boardGame_server.Communication;

public class InternetMultiplayerController : IMultiplayerViewController, IMultiplayerPlayerInputController
{
    public List<Tuple<Player, string>> FetchRequestedGameLobbiesWithLobbyNameAndPlayer()
    {
        var newGames = new List<Tuple<Player, string>>();
        // TODO: Return all the new wanted games
        return newGames;
    }

    public List<Input> FetchPlayerInputs(int gameID)
    {
        var playerInputs = new List<Core.Input>();
        // TODO: Return all the inputs from the game with the given gameID
        return playerInputs;
    }

    public void SendNewGameStateToPlayers(GameState state)
    {
        // TODO: Send game state to the players in the game connected to the game state input
    }
}