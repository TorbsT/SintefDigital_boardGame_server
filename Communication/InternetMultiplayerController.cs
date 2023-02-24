using Core;
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using Microsoft.AspNetCore.Mvc;
using Newtonsoft.Json;

namespace Communication;

[ApiController]
[Route("[controller]")]
public class InternetMultiPlayerInfoController : ControllerBase, IMultiPlayerInfoViewController, IMultiPlayerInfoPlayerInfoInputController
{
    private ReaderWriterLockSlim _lock = new ReaderWriterLockSlim();
    private GameState gamestate = new GameState("example", 42);

    public List<(PlayerInfo, string)> FetchRequestedGameLobbiesWithLobbyNameAndPlayerInfo()
    {
        VerifyLock();
        var newGames = new List<(PlayerInfo, string)>();
        // TODO: Return all the new wanted games
        return newGames;
    }

    public List<Input> FetchPlayerInfoInputs(int gameID)
    {
        VerifyLock();
        var PlayerInfoInputs = new List<Input>();
        // TODO: Return all the inputs from the game with the given gameID
        return PlayerInfoInputs;
    }

    public void SendNewGameStateInfoToPlayerInfos(GameStateInfo state)
    {
        VerifyLock();
        // TODO: Send game state to the PlayerInfos in the game connected to the game state input
    }

    public void Lock()
    {
        _lock.EnterWriteLock();
    }

    public void ReleaseLock()
    {
        _lock.ExitReadLock();
    }

    public void VerifyLock()
    {
        if (!_lock.IsWriteLockHeld) throw new InvalidOperationException("Before making any calls to this object " +
                                                                        "it needs to be locked unsing Lock() and " +
                                                                        "needs to be released once done!");
    }

    [Route("gamestate/{id}")]
    [HttpGet]
    public ActionResult<GameState> GetGamestate(int id)
    {
        /*
         TODO: authenticate other players by comparing the player ID to the IDs of the connected players
               return gamestate if player is successfully authenticated
         */
        if (id == 1)
        {
            return Ok(JsonConvert.SerializeObject(gamestate));
        }
        else
        {
            return Unauthorized("Error(401): Authentication failed");
        }
    }
    [HttpPost]
    [Route("gamestate/{id}")]
    public IActionResult UpdateGameState(int id, [FromBody] GameState gamestate = default)
    {
        /*
         TODO: Validate gamestate parameter (Creating a static method for validation is probably the best approach for this)
               Validate the player sending the POST request
               Return 401(Unauthorized) if gamestate is valid but player is not in the session or if it's not the player's turn
               Make sure the method only returns 200(OK) if the gamestate is valid, the player is in the session, and it's the player's turn
         */
        if (gamestate.Equals(default(GameState)))
        {
            return BadRequest("Error(400): Bad Request");
        }
        if (id == 1)
        {
            return Ok(JsonConvert.SerializeObject(gamestate));
        }
        else
        {
            return Unauthorized("Error(401): Authentication failed");
        }
    }
}