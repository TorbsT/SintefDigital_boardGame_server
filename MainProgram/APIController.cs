using Communication;
using Core;
using Logging;
using Microsoft.AspNetCore.Mvc;
using Newtonsoft.Json;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace MainProgram
{
    [ApiController]
    [Route("[controller]")]
    public class APIController : ControllerBase
    {
        private readonly IGameControllerService _gameController;

        public APIController(IGameControllerService service)
        {
            _gameController = service;
        }


        [Route("")]
        [HttpGet]
        public ActionResult<PlayerInfo> test()
        {
            PlayerInfo p1 = new PlayerInfo();
            p1.UniqueID = 12345;
            p1.Name = "67890";
            return Ok(JsonConvert.SerializeObject(new WantedLobbyInfo(){PlayerInfo = p1, LobbyName = "bruh"}));
        }

        [Route("create/game")]
        [HttpPost]
        public ActionResult<GameState> CreateGameAndAssignHost([FromBody] WantedLobbyInfo playerInfoAndLobbyName)
        {
            //curl -X POST -H "Content-Type: application/json" -d "{\"Item1\":{\"ConnectedGameID\":1,\"InGameID\":2,\"UniqueID\":3,\"Name\":\"John\",\"Position\":{\"ID\":4,\"Name\":\"PositionName\"}},\"Item2\":\"bruh\"}" localhost:5000/API
            try
            {
                var game = _gameController.CreateNewGame(playerInfoAndLobbyName);

                if (game.PlayerInfos.Any(p => p.UniqueID == playerInfoAndLobbyName.PlayerInfo.UniqueID))
                    return Ok(JsonConvert.SerializeObject(game));
            }
            catch (Exception)
            {
                
            }
            
            return NotFound($"Failed to get the new game state");
        }

        [Route("create/playerID")]
        [HttpGet]
        public ActionResult<int> GetUniquePlayerID()
        {
            return _gameController.MakeNewPlayerID();
        }

        [Route("debug/playerIDs/amount")]
        [HttpGet]
        public ActionResult<int> GetAmountOfUniquePlayerIDs()
        {
            return _gameController.GetAmountOfCreatedPlayerIDs();
        }

        [Route("games/{id}")]
        [HttpGet]
        public ActionResult<GameState> GetGamestate(int id)
        {
            /*
             TODO: authenticate other players by comparing the player ID to the IDs of the connected players
                   return gamestate if player is successfully authenticated
             */
            var games = _gameController.GetGameStateInfos();

            if (games.All(g => g.ID != id)) return NotFound($"There is no game with {id}");
            return Ok(JsonConvert.SerializeObject(games.Find(g => g.ID == id)));
        }
        
        [HttpPost]
        [Route("games/input")]
        public ActionResult<GameStateInfo> HandlePlayerInput([FromBody] Input input)
        {
            /*
             TODO: Validate gamestate parameter (Creating a static method for validation is probably the best approach for this)
                   Validate the player sending the POST request
                   Return 401(Unauthorized) if gamestate is valid but player is not in the session or if it's not the player's turn
                   Make sure the method only returns 200(OK) if the gamestate is valid, the player is in the session, and it's the player's turn
             */
            
            var games = _gameController.GetGameStateInfos();

            if (games.All(g => g.ID != input.PlayerInfo.ConnectedGameID)) return NotFound($"There is no game with {input.PlayerInfo.ConnectedGameID}");
            try
            {
                var game = _gameController.HandlePlayerInput(input);
                return Ok(JsonConvert.SerializeObject(game));
            }
            catch (Exception e)
            {
                return Problem($"Failed to handle input because {e}");
            }
        }
    }
}
