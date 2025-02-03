
let addressArray = []
let callCounters = 0
let priceResult = undefined

async function _fetchBalancesOfAddressesFromRpc(addresses) {
    const rpcUrl = "RPC_NODE_URL";

    const batchRequest = addresses.map((address, index) => ({
        jsonrpc: "2.0",
        id: index + 1,
        method: "eth_getBalance",
        params: [address, "latest"]
    }));

    try {
        const response = await fetch(rpcUrl, {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify(batchRequest)
        });

        const data = await response.json();

        let balances = {}

        data.forEach((item, i) => {
            balances[addresses[i]] = item.result ? Number(BigInt(item.result)) / 1e18 : null
        });

        return balances;
    } catch (error) {
        console.error("Error fetching balances:", error);
        return null;
    }
}


async function fetchBalance(address) { 
    callCounters++
    addressArray.push(address);

    setImmediate(async ()=> { 
        callCounters--;
        if (callCounters === 0 && priceResult === undefined) {
            priceResult = await _fetchBalancesOfAddressesFromRpc(addressArray);
        }
    });
    
    return new Promise(async (resolve)=> {
        while (priceResult === undefined) { 
            await new Promise((resolveInterior)=> setTimeout(resolveInterior, 100));
        }

        resolve(priceResult[address])
    })
}

const main = async () => { 
    const addresses = [
        "0xF7B31119c2682c88d88D455dBb9d5932c65Cf1bE",
        "0x3CBdeD43EFdAf0FC77b9C55F6fC9988fCC9b757d",
        "0x53b6936513e738f44FB50d2b9476730C0Ab3Bfc1",
        "0x72a5843cc08275C8171E582972Aa4fDa8C397B2A",
        "0x1da5821544e25c636c1417Ba96Ade4Cf6D2f9B5A",
      ];
    console.log(addresses)
    const balances = await Promise.all(addresses.map(fetchBalance));
    console.log(balances)
}

main()

