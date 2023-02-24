using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Core;

internal class GameState
{
    private GameStateInfo gameStateInfo;


    //Updates internal gamestate
    public void UpdateGameStateInfo(GameStateInfo updatedGameStateInfo)
    {
        this.gameStateInfo = updatedGameStateInfo;
    }

    //Returns internal gamestate
    public GameStateInfo GetGameStateInfo()
    {
        return this.gameStateInfo;
    }
}
