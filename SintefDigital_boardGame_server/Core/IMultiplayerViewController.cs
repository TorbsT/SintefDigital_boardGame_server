namespace SintefDigital_boardGame_server.Core;

public interface IMultiplayerViewController : ILocker
{
    void SendNewGameStateToPlayers(GameState state);
}