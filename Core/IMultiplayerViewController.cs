namespace Core;

public interface IMultiPlayerInfoViewController : ILocker
{
    void SendNewGameStateInfoToPlayerInfos(GameStateInfo state);
}