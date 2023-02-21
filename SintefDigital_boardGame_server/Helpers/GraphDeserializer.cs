using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace SintefDigital_boardGame_server.Helpers
{
    internal class GraphDeserializer
    {
        private static readonly string neighboursKey = "neigs";
        // LATER private static readonly string zoneKey;

        // An example of a serialized graph:
        // neigs:1,2; neigs:; neigs:0;
        // represents a graph with the following edges:
        // (0, 1), (0, 2), (2, 0)
        // node 0 and 2 are connected bidirectionally,
        // and there is a one-directional edge from 0 to 1.
        public static Graph Deserialize(string input)
        {
            if (input == null) throw new ArgumentException
                    ($"null is not a valid input to deserialize");
            // args = arg1; arg2; arg3
            string[] args = input.Split(";");

            Graph graph = new();
            int nodeId = 0;
            foreach (string arg in args)
            {
                // keyValuePairs = keyValue1 keyValue2 keyValue3
                string[] keyValuePairs = arg.Trim().Split(" ");
                foreach (string keyValue in keyValuePairs)
                {
                    // key:value = keyValue
                    string[] tempArr = keyValue.Split(":");
                    if (tempArr.Length != 2) throw new ArgumentException
                            ($"{input} is not a valid input to deserialize");
                    string key = tempArr[0];
                    string value = tempArr[1];

                    if (key == neighboursKey)
                    {
                        // neigs:0,5,6,8
                        string[] neighbourIdStrings = value.Split(",");
                        foreach (string neighbourIdString in neighbourIdStrings)
                        {
                            if (neighbourIdString == "") continue;
                            if (int.TryParse(neighbourIdString, out int neighbourId))
                            {
                                graph.BuildEdge(nodeId, neighbourId);
                            } else
                            {
                                throw new ArgumentException
                                    ($"{keyValue} is not a valid input because " +
                                    $"{neighbourIdString} is not an integer");
                            }
                        }
                    } else
                    {
                        throw new ArgumentException
                            ($"{key} is an invalid argument in {input}");
                    }
                }
                nodeId++;
            }
            return graph;
        }
    }
}
