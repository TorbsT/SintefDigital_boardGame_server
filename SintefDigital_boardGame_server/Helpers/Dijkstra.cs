// The following piece of code is copied from TorbuTils
// https://github.com/TorbsT/Torbutils/blob/main/Packages/com.torbst.torbutils/Runtime/Giraphe/Dijkstra.cs
using System.Collections;

namespace SintefDigital_boardGame_server.Helpers
{
    [Serializable]
    public class Dijkstra
    {
        public event Action? Done;
        private readonly Graph inputGraph;
        private readonly int startId;
        private readonly int maxDistance;
        public Graph ResultTree { get; private set; }
        public Dijkstra(Graph inputGraph, int startId, int maxDistance = int.MaxValue)
        {
            this.inputGraph = inputGraph;
            this.startId = startId;
            this.maxDistance = maxDistance;
            ResultTree = Graph.MakeFromNodeInfo(inputGraph);
        }
        public IEnumerable Solve()
        {
            Queue<int> queue = new();  // ids
            queue.Enqueue(startId);
            ResultTree.SetNodeInfo(startId, Settings.CostInfoKey, 0);

            while (queue.Count > 0)
            {
                int current = queue.Dequeue();
                int? ch = (int?)ResultTree.GetNodeInfo(current, Settings.CostInfoKey);
                int costHere = ch == null ? 0 : ch.Value;
                foreach (int next in inputGraph.CopyEdgesFrom(current))
                {
                    yield return null;
                    int? currentNextCost = inputGraph.GetWeight(current, next);
                    int hypoCost = costHere + (int)currentNextCost;
                    if (hypoCost > maxDistance) continue;
                    int? prevCost = (int?)ResultTree.GetNodeInfo(next, Settings.CostInfoKey);
                    if (prevCost == null || hypoCost < prevCost)
                    {
                        if (prevCost != null)
                        {
                            foreach (int backtrack in ResultTree.CopyEdgesTo(next))
                            {
                                ResultTree.RemoveEdge(backtrack, next);
                            }
                        }

                        ResultTree.BuildEdge(current, next);
                        ResultTree.SetNodeInfo(next, Settings.CostInfoKey, hypoCost);
                        queue.Enqueue(next);
                    }
                }
            }
            Done?.Invoke();
        }
    }
}