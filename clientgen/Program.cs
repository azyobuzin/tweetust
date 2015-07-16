using System.IO;
using System.Linq;
using RestApisGen;

namespace Tweetust.ClientGen
{
    class Program
    {
        static void Main(string[] args)
        {
            var apis = Directory.GetFiles(Path.Combine("clientgen", "CoreTweet", "ApiTemplates"))
                .Where(x => !x.Contains("test.api"))
                .Select(ApiParent.Parse);

            using (var writer = new StreamWriter(Path.Combine("src", "clients.rs")))
            {
                new ClientRsGen(apis, writer).Generate();
            }
        }
    }
}
