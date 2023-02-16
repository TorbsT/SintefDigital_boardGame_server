namespace SintefDigital_boardGame_server.Core;

public interface IMultiplayerGameController
{
    void SendNewGameStateToPlayers(GameState state);
}