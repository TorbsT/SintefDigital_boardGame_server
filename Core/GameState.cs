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
    //private List<Player> _players;
    private List<Node> _nodemap;

    public GameState(string gameName, int gameID)
    {
        _gameStateInfo = new GameStateInfo();
        _gameStateInfo.Name = gameName;
        _gameStateInfo.ID = gameID;
        _gameStateInfo.PlayerInfos = new List<PlayerInfo>();
        //_players = new List<Player>();
        this._nodemap = NodeMap.Map();
    }

    //Updates internal gamestate
    public void UpdateGameStateInfo(GameStateInfo updatedGameStateInfo)
    {
        UpdatePlayersBasedOnInfos(updatedGameStateInfo.PlayerInfos);
    }

    //Returns internal gamestate
    public GameStateInfo GetGameStateInfo()
    {
        return _gameStateInfo;
    }

    public void AssignPlayerToGame(PlayerInfo playerInfo)
    {
        if (_gameStateInfo.PlayerInfos.Any(player => player.UniqueID == playerInfo.UniqueID)) throw new ArgumentException("This player already exists in this game");
        playerInfo.ConnectedGameID = _gameStateInfo.ID;
        _gameStateInfo.PlayerInfos.Add(playerInfo);
    }

    public void UpdatePlayersBasedOnInfos(List<PlayerInfo> playerInfos)
    {
        foreach (PlayerInfo playerInfo in playerInfos)
        {
            if (_gameStateInfo.PlayerInfos.Any(player => player.UniqueID == playerInfo.UniqueID))
            {
                _gameStateInfo.PlayerInfos.RemoveAll(player => player.UniqueID == playerInfo.UniqueID);
                _gameStateInfo.PlayerInfos.Add(playerInfo);
            }
        }
    }

    internal bool ContainsUniquePlayerID(PlayerInfo playerInfo)
    {
        return _gameStateInfo.PlayerInfos.Any(player => player.UniqueID.Equals(playerInfo.UniqueID));
    }

    public static implicit operator GameStateInfo(GameState gameState)
    {
        return gameState._gameStateInfo;
    }
}
