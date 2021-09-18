import { AppWebsocket } from '@holochain/conductor-api';

const HOST = "localhost";
const APP_PORT = 8888;
const ADMIN_PORT = 65000;
const HAPP_ID = 'sample-happ';

export class AppClient {
    #host = '';
    #appPort = '';
    #appClient = null;
    #cellId = [];
    #agentPubKey = '';

    constructor() {
        // we could use the consts directly instead of these assignments
        // it's only for when they'll be in a different module or coming from env vars
        this.#host = HOST || 'localhost';
        this.#adminPort = ADMIN_PORT;
        this.#appPort = APP_PORT || '8888';
    }

    async connect() {
        const adminClient = await AdminWebsocket.connect(`ws://${this.#host}:${this.#adminPort}`);
        const installedHApps = await adminClient.listApps({ status_filter: {} });
        if (installedHApps.length) {
            // hApp is already installed -> activate it
            const response = await adminClient.activateApp({ installed_app_id: HAPP_ID });
            this.#cellId = response.app.cell_data[0].cell_id;
        } else {
            // install hApp first with a generated agent pub key
            this.#agentPubKey = await adminClient.generateAgentPubKey();
            const response = await adminClient.installAppBundle({ path: 'zome/workdir/happ/sample-happ.happ', agent_key: this.#agentPubKey, membrane_proofs: {} });
            this.#cellId = response.cell_data[0].cell_id;
            await adminClient.attachAppInterface({ port: this.#appPort });
        }
        // doesn't work in the browser
        // adminClient.client.close();
        adminClient.client.socket.close();
        this.#appClient = await AppWebsocket.connect(`ws://${this.#host}:${this.#appPort}`);
    }

    async close() {
        // same, client.close() doesn't work in the browser
        await this.#appClient.client.socket.close();
    }

    async startNewGame() {
        const gameCode = Math.random().toString(36).substr(2, 6).toUpperCase();
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'create_game_code_anchor',
            provenance: this.#agentPubKey,
            payload: gameCode
        };
        return this.#appClient.callZome(params);
    }

    async getPlayers(gameCode) {
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'get_players_for_game_code',
            provenance: this.#agentPubKey,
            payload: gameCode
        };
        return this.#appClient.callZome(params);
    }


    async joinGame(gameCode, nickname) {
        console.log("joinGame:", gameCode, nickname);
        let payload = { gamecode: gameCode, nickname: nickname };
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'join_game_with_code',
            provenance: this.#agentPubKey,
            payload: payload,
        };
        return this.#appClient.callZome(params);
    }
    async startGame(gameCode) {
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'start_game_session_with_code',
            provenance: this.#agentPubKey,
            payload: gameCode
        };
        return this.#appClient.callZome(params);
    }
    async currentRoundForGameCode(gameCode) {
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'current_round_for_game_code',
            provenance: this.#agentPubKey,
            payload: gameCode
        };
        return this.#appClient.callZome(params);
    }
    async currentRoundInfoForHeaderHash(headerHash) {
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'current_round_info',
            provenance: this.#agentPubKey,
            payload: headerHash
        };
        return this.#appClient.callZome(params);
    }
    async makeMove(amount, prev_round_hash) {
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'make_new_move',
            provenance: this.#agentPubKey,
            payload: { resource_amount: parseInt(amount), previous_round: prev_round_hash }
        };
        return this.#appClient.callZome(params);
    }
    async tryCloseRound(prev_round_hash) {
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'try_to_close_round',
            provenance: this.#agentPubKey,
            payload: prev_round_hash
        };
        return this.#appClient.callZome(params);
    }


    async getMyOwnedSessions(amount, prev_round_hash) {
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'get_my_owned_sessions',
            provenance: this.#agentPubKey,
            payload: null
        };
        return this.#appClient.callZome(params);
    }
    async getMyPlayedSessions(amount, prev_round_hash) {
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'get_my_played_sessions',
            provenance: this.#agentPubKey,
            payload: null
        };
        return this.#appClient.callZome(params);
    }
    async getAllMySessions(amount, prev_round_hash) {
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'get_all_my_sessions',
            provenance: this.#agentPubKey,
            payload: null
        };
        return this.#appClient.callZome(params);
    }
    async getMyActiveSessions(amount, prev_round_hash) {
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'get_my_active_sessions',
            provenance: this.#agentPubKey,
            payload: null
        };
        return this.#appClient.callZome(params);
    }

}

function signalHandler(connection, signal) {
    // ignore signals not meant for me
    // if (!connection.syn || bufferToBase64(signal.data.cellId[1]) != connection.syn.me) {
    //     return
    // }
    console.log('Got Signal', signal.data.payload.signal_name, signal)
    alert(signal.data.payload.signal_name);
    switch (signal.data.payload.signal_name) {
        case 'SyncReq':
            connection.session.syncReq({ from: signal.data.payload.signal_payload })
            break
        default:
            console.log("signal data: {:?}", signal.data);
    }
}
