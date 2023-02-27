using System;
using System.Collections.Generic;
using System.Linq;
using System.Security.Cryptography.X509Certificates;
using System.Text;
using System.Threading.Tasks;

namespace Core;

public class GameState //TODO: protect later
{
    private GameStateInfo _gameStateInfo;
    private List<Player> _players;

    public GameState(string gameName, int gameID)
    {
        _gameStateInfo = new GameStateInfo();
        _gameStateInfo.Name = gameName;
        _gameStateInfo.ID = gameID;
        _gameStateInfo.PlayerInfos = new List<PlayerInfo>();
        _players = new List<Player>();
    }

    //Updates internal gamestate
    public void UpdateGameStateInfo(GameStateInfo updatedGameStateInfo)
    {
        UpdatePlayersBasedOnInfos(updatedGameStateInfo.PlayerInfos);
    }

    //Returns internal gamestate
    public GameStateInfo GetGameStateInfo()
    {
        UpdateGameStateInfoPlayerInfos();
        return _gameStateInfo;
    }

    public void AssignPlayerToGame(PlayerInfo playerInfo)
    {
        if (_players.Any(player => player.RetrievePlayerInfo().UniqueID == playerInfo.UniqueID)) throw new ArgumentException("This player already exists in this game");
        _players.Add(new Player(playerInfo));
    }

    public void UpdatePlayersBasedOnInfos(List<PlayerInfo> playerInfos)
    {
        foreach (PlayerInfo playerInfo in playerInfos)
        {
            if (_players.Any(player => player.RetrievePlayerInfo().UniqueID == playerInfo.UniqueID))
            {
                _players.First(player => player.RetrievePlayerInfo().UniqueID == playerInfo.UniqueID).UpdatePlayer(playerInfo);
            }
        }
    }

    public static implicit operator GameStateInfo(GameState gameState)
    {
        return gameState._gameStateInfo;
    }

    private void UpdateGameStateInfoPlayerInfos()
    {
        List<PlayerInfo> playerInfos = new List<PlayerInfo>();
        foreach (var player in _players) playerInfos.Add(player.RetrievePlayerInfo());
        _gameStateInfo.PlayerInfos = playerInfos;
    }
}
