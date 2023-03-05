using Core;
using Logging;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.AspNetCore.Mvc.Controllers;

namespace MainProgram
{

    public interface IGameControllerService
    {
        public List<GameStateInfo> GetGameStateInfos();
        public int MakeNewPlayerID();
        public GameStateInfo CreateNewGame(WantedLobbyInfo lobbyNameAndPlayerInfo);
        public GameStateInfo HandlePlayerInput(Input input);
        public int GetAmountOfCreatedPlayerIDs();
    }
    
    public class GameControllerService : IGameControllerService
    {
        private readonly GameController _gameController;

        public GameControllerService()
        {
            _gameController = new GameController(new ThresholdLogger(LogLevel.Debug, LogLevel.Ignore));
        }

        public List<GameStateInfo> GetGameStateInfos()
        {
            var info = new List<GameStateInfo>();
            lock (_gameController) info = _gameController.GetGameStateInfos();
            return info;
        }

        public int MakeNewPlayerID()
        {
            int i;
            lock (_gameController) i = _gameController.MakeNewPlayerID();
            return i;
        }

        public GameStateInfo CreateNewGame(WantedLobbyInfo lobbyNameAndPlayerInfo)
        {
            GameStateInfo info;
            lock (_gameController) info = _gameController.CreateNewGame(lobbyNameAndPlayerInfo);
            return info;
        }

        public GameStateInfo HandlePlayerInput(Input input)
        {
            GameStateInfo game;
            lock (_gameController) game = _gameController.HandlePlayerInput(input);
            return game;
        }

        public int GetAmountOfCreatedPlayerIDs()
        {
            int amount;
            lock (_gameController) amount = _gameController.GetAmountOfCreatedPlayerIDs();
            return amount;
        }
    }
    
    public class Startup
    {
        
        public Startup(IConfiguration configuration)
        {
            Configuration = configuration;
        }

        public IConfiguration Configuration { get; }

        // This method gets called by the runtime. Use this method to add services to the container.
        public void ConfigureServices(IServiceCollection services)
        {
            services.AddSingleton<IGameControllerService, GameControllerService>();
            
            services.AddControllers().ConfigureApplicationPartManager(apm =>
            {
                var feature = new ControllerFeature();
                apm.PopulateFeature(feature);
                Console.WriteLine("Controller count: " + feature.Controllers.Count);
                foreach (var controller in feature.Controllers)
                {
                    Console.WriteLine(controller.FullName);
                }
            });
        }

        // This method gets called by the runtime. Use this method to configure the HTTP request pipeline.
        public void Configure(IApplicationBuilder app, IWebHostEnvironment env)
        {
            Console.WriteLine("Configuring controllers...");
            if (env.IsDevelopment())
            {
                app.UseDeveloperExceptionPage();
            }

            app.UseRouting();

            app.UseAuthorization();

            app.UseEndpoints(endpoints =>
            {
                endpoints.MapControllers();
            });
        }
    }
}
