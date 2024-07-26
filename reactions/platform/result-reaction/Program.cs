using Dapr;
using Dapr.Actors.Client;
using Dapr.Client;
using Microsoft.AspNetCore.Components;
using System.Net.Http;
using Microsoft.Extensions.DependencyInjection;
using result_reaction.Services;
using System.Text.Json;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

var builder = WebApplication.CreateBuilder(args);
var configuration = BuildConfiguration();

var pubsubName = configuration.GetValue<string>("PubsubName", "rg-pubsub");
var configDirectory = configuration.GetValue<string>("QueryConfigPath", "/etc/queries");
var queryContainerId = configuration.GetValue<string>("QueryContainer", "default");



builder.Services.AddDaprClient();
builder.Services.AddActors(x => { });
builder.Services.AddSingleton<IResultViewClient, ResultViewClient>();


var app = builder.Build();
app.UseRouting();
app.UseCloudEvents();



app.Urls.Add("http://0.0.0.0:80");  //dapr
app.Urls.Add("http://0.0.0.0:8080"); //app

// Get current result set
app.MapGet("/{queryId}", async (string queryId) => 
{
    Console.WriteLine("Retrieving the current result set");
    var resultViewClient = app.Services.GetRequiredService<IResultViewClient>();
    List<JsonElement> result = new List<JsonElement>();
    await foreach (var item in resultViewClient.GetCurrentResult(queryContainerId, queryId))
    {
        var element = item.RootElement;
        if (element.TryGetProperty("data", out var data))
        {
            result.Add(data);
        }
    }
    Console.WriteLine("Result:" + result.Last());
    return result.Last();
});

// Adding an endpoint that supports retrieving all results
app.MapGet("/{queryId}/all", async (string queryId) => 
{
    Console.WriteLine("Getting all results");
    var resultViewClient = app.Services.GetRequiredService<IResultViewClient>();
    Console.WriteLine("Current result");
    List<JsonElement> result = new List<JsonElement>();
    await foreach (var item in resultViewClient.GetCurrentResult(queryContainerId, queryId))
    {
        var element = item.RootElement;
        if (element.TryGetProperty("data", out var data))
        {
            result.Add(data);
        }
    }
    Console.WriteLine("Result:" + result);
    return result;
});

// Get a result set at a specific timestamp
// Ts is in the format of "2023-04-20T00:00:00"
app.MapGet("/{queryId}/{ts}", async (string queryId, string ts) => 
{
    var resultViewClient = app.Services.GetRequiredService<IResultViewClient>();
    Console.WriteLine($"Result set at {ts}:");
    List<JsonElement> result = new List<JsonElement>();
    await foreach (var item in resultViewClient.GetCurrentResultAtTimeStamp(queryContainerId, queryId,ts))
    {
        var element = item.RootElement;
        if (element.TryGetProperty("data", out var data))
        {
            result.Add(data);
        }
    }
    Console.WriteLine("Result:" + result);
    return result;
});
app.Run();



static IConfiguration BuildConfiguration()
{
    return new ConfigurationBuilder()
        .SetBasePath(Directory.GetCurrentDirectory())
        .AddJsonFile("appsettings.json", optional: true, reloadOnChange: true)
        .AddEnvironmentVariables()
        .Build();
}
