import asyncio
from raftify import RaftServiceClient, ConfChangeRequest, ConfChangeSingle, ConfChangeType

from ..state_machine import SetCommand


# Run "python -m examples.client.main" to execute this script.
async def main() -> None:
    """
    A simple set of commands to test and show usage of RaftServiceClient.
    Please bootstrap the Raft cluster before running this script.
    """

    client = await RaftServiceClient.build("127.0.0.1:60061")
    await client.propose(SetCommand("1", "A").encode())

    peers_json = await client.get_peers()
    print("Peers: ", peers_json)

    addNode = ConfChangeSingle()
    addNode.set_change_type(ConfChangeType.AddNode)
    addNode.set_node_id(3)
    removeNode = ConfChangeSingle()
    removeNode.set_change_type(ConfChangeType.RemoveNode)
    removeNode.set_node_id(9)
    confChangeRequest = ConfChangeRequest([addNode], ["127.0.0.1:60069"])
    conf_change_result = await client.change_config(confChangeRequest)
    print("ConfChangeResult ", conf_change_result)

    peers_json = await client.get_peers()
    print("Peers: ", peers_json)

if __name__ == "__main__":
    asyncio.run(main())
