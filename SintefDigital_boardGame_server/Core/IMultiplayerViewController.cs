namespace SintefDigital_boardGame_server.Core;

public interface IMultiplayerViewController
{
    void SendNewGameStateToPlayers(GameState state);
}