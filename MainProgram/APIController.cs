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
    public class APIController : ControllerBase, IDisposable
    {
        private static readonly GameState gamestate = new GameState("example", 42);
        private static readonly ThresholdLogger thresholdLogger = new ThresholdLogger(LogLevel.Debug, LogLevel.Ignore);
        private static readonly InternetMultiPlayerInfoController internetMultiPlayerInfoController = new InternetMultiPlayerInfoController();
        private static GameController gameController;

        private static readonly Lazy<APIController> _instance = new Lazy<APIController>(() => new APIController());

        public static APIController Instance => _instance.Value;
        
        public APIController()
        {
            gameController = new GameController(thresholdLogger, internetMultiPlayerInfoController, internetMultiPlayerInfoController);
            gameController.Run();
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

        [Route("")]
        [HttpPost]
        public ActionResult<GameState> CreateGameAndAssignHost([FromBody] WantedLobbyInfo playerInfoAndLobbyName)
        {
            //curl -X POST -H "Content-Type: application/json" -d "{\"Item1\":{\"ConnectedGameID\":1,\"InGameID\":2,\"UniqueID\":3,\"Name\":\"John\",\"Position\":{\"ID\":4,\"Name\":\"PositionName\"}},\"Item2\":\"bruh\"}" localhost:5000/API
            internetMultiPlayerInfoController.Lock();
            internetMultiPlayerInfoController.AddNewWantedGameLobby(playerInfoAndLobbyName);
            internetMultiPlayerInfoController.ReleaseLock();

            Thread.Sleep(10);
            
            internetMultiPlayerInfoController.Lock();
            var game = internetMultiPlayerInfoController.FetchGameWithPlayer(playerInfoAndLobbyName.PlayerInfo);
            internetMultiPlayerInfoController.ReleaseLock();

            Console.WriteLine(playerInfoAndLobbyName.PlayerInfo.UniqueID);
            
            int counter = 0;
            
            while (game.PlayerInfos.All(p => p.UniqueID != playerInfoAndLobbyName.PlayerInfo.UniqueID) && counter < 1_000)
            {
                internetMultiPlayerInfoController.Lock();
                game = internetMultiPlayerInfoController.FetchGameWithPlayer(playerInfoAndLobbyName.PlayerInfo);
                internetMultiPlayerInfoController.ReleaseLock();
                Thread.Sleep(10);
                counter++;
            }

            if (game.PlayerInfos.Any(p => p.UniqueID == playerInfoAndLobbyName.PlayerInfo.UniqueID))
                return Ok(JsonConvert.SerializeObject(game));
            
            return NotFound($"Failed to get the game state, but it might come later");
        }

        [Route("playerID")]
        [HttpGet]
        public ActionResult<int> GetUniquePlayerID()
        {
            internetMultiPlayerInfoController.Lock();
            internetMultiPlayerInfoController.NotifyWantID();
            var (gotId, id) = internetMultiPlayerInfoController.FetchUniqueID();
            internetMultiPlayerInfoController.ReleaseLock();
            while (gotId == false)
            {
                internetMultiPlayerInfoController.Lock();
                (gotId, id) = internetMultiPlayerInfoController.FetchUniqueID();
                internetMultiPlayerInfoController.ReleaseLock();
                Thread.Sleep(10);
            }
            return id;
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

        public void Dispose()
        {
            gameController.Dispose();
        }
    }
}
