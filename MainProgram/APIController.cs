using Core;
using Microsoft.AspNetCore.Mvc;
using Newtonsoft.Json;

namespace MainProgram
{
    [ApiController]
    [Route("[controller]")]
    public class APIController : ControllerBase
    {
        private GameState gamestate = new GameState("example", 42);

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
}
