------------------------------- MODULE ICS02 ----------------------------------

EXTENDS Integers, FiniteSets, IBCDefinitions

\* retrieves `clientId`'s data
ICS02_GetClient(clients, clientId) ==
    clients[clientId]

\* check if `clientId` exists
ICS02_ClientExists(clients, clientId) ==
    ICS02_GetClient(clients, clientId).heights /= AsSetInt({})

\* update `clientId`'s data
ICS02_SetClient(clients, clientId, client) ==
    [clients EXCEPT ![clientId] = client]

ICS02_CreateClient(chain, chainId, height) ==
    LET action == AsAction([
        type |-> "ICS02CreateClient",
        chainId |-> chainId,
        clientState |-> height,
        consensusState |-> height
    ]) IN
    \* check if the client exists (it shouldn't)
    IF ICS02_ClientExists(chain.clients, chain.clientIdCounter) THEN
        \* if the client to be created already exists,
        \* then there's an error in the model
        [
            clients |-> chain.clients,
            clientIdCounter |-> chain.clientIdCounter,
            action |-> action,
            outcome |-> "ModelError"
        ]
    ELSE
        \* if it doesn't, create it
        LET client == [
            heights|-> {height}
        ] IN
        \* return result with updated state
        [
            clients |-> ICS02_SetClient(
                chain.clients,
                chain.clientIdCounter,
                client
            ),
            clientIdCounter |-> chain.clientIdCounter + 1,
            action |-> action,
            outcome |-> "ICS02CreateOK"
        ]

ICS02_UpdateClient(chain, chainId, clientId, height) ==
    LET action == AsAction([
        type |-> "ICS02UpdateClient",
        chainId |-> chainId,
        clientId |-> clientId,
        header |-> height
    ]) IN
    \* check if the client exists
    IF ~ICS02_ClientExists(chain.clients, clientId) THEN
        \* if the client does not exist, then set an error outcome
        [
            clients |-> chain.clients,
            action |-> action,
            outcome |-> "ICS02ClientNotFound"
        ]
    ELSE
        \* if the client exists, check its height
        LET client == ICS02_GetClient(chain.clients, clientId) IN
        LET highestHeight == Max(client.heights) IN
        IF highestHeight >= height THEN
            \* if the client's new height is not higher than the highest client
            \* height, then set an error outcome
            [
                clients |-> chain.clients,
                action |-> action,
                outcome |-> "ICS02HeaderVerificationFailure"
            ]
        ELSE
            \* if the client's new height is higher than the highest client
            \* height, then update the client
            LET updatedClient == [client EXCEPT
                !.heights = client.heights \union {height}
            ] IN
            \* return result with updated state
            [
                clients |-> ICS02_SetClient(
                    chain.clients,
                    clientId,
                    updatedClient
                ),
                action |-> action,
                outcome |-> "ICS02UpdateOK"
            ]

===============================================================================
