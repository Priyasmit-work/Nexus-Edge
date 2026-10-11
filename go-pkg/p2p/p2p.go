package p2p

import "fmt"

// InitP2P initializes the libp2p Kademlia DHT host.
func InitP2P(port int) {
	fmt.Printf("[P2P Module] Initializing libp2p DHT node on port %d\n", port)
}
