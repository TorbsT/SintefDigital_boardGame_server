using System.Net;
using Microsoft.AspNetCore.Mvc.Testing;
using Xunit;

using MainProgram;

namespace Tests.MainProgram;

public class APIControllerTest : IClassFixture<WebApplicationFactory<Startup>>
{
    private readonly HttpClient _httpClient;

    public APIControllerTest(WebApplicationFactory<Startup> factory)
    {
        _httpClient = factory.CreateClient();
    }

    [Theory]
    [InlineData(0)]
    [InlineData(1)]
    [InlineData(5)]
    [InlineData(50)]
    [InlineData(500)]
    [InlineData(5000)]
    [InlineData(50000)]
    public async Task GetUniqueIDs(int amount)
    {
        var ids = new List<int>();

        for (var _ = 0; _ < amount; _++)
        {
            var response = await _httpClient.GetAsync("API/create/playerID");
            Assert.Equal(HttpStatusCode.OK, response.StatusCode);
            int id = Int32.Parse(await response.Content.ReadAsStringAsync());
            Assert.DoesNotContain(id, ids);
        }
    }
    
    
    
}