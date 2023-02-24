using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Core;

internal class Player
{
    private PlayerInfo _playerInfo;

    public Player(PlayerInfo playerInfoStruct)
    {
        _playerInfo = playerInfoStruct;
    }

    public PlayerInfo RetrievePlayerInfo()
    {
        return _playerInfo;
    }

    public void UpdatePlayer(PlayerInfo player)
    {
        if (_playerInfo.UniqueID != player.UniqueID) throw new ArgumentException("The players do not have the same unique ID");
        _playerInfo = player;
    }

    public static implicit operator PlayerInfo(Player player)
    {
        return player._playerInfo;
    }
}
