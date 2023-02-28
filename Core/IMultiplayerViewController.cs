namespace Core;

public interface IMultiPlayerInfoViewController : ILocker
{
    void SendNewGameStateInfoToPlayerInfos(GameStateInfo state);
    void HandleGeneratedUniqueIDs(List<int> uniqueIDs);

}