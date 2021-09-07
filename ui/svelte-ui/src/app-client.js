import { AppWebsocket } from '@holochain/conductor-api';
import { bufferToBase64, encodeJson } from './utils';

// hc sandbox generate workdir/happ/ --run=8888 --app-id tragedy

export class AppClient {
    #host = '';
    #appPort = '';
    #appClient = null;
    #cellId = '';
    #dna = '';
    #dnaStr = '';
    #agentPubKey = '';
    #appId = "tragedy_of_commons";
    #appInfo = null;

    constructor(host, appPort) {
        this.#host = host || 'localhost';
        this.#appPort = appPort || '8888';
    }
    async connect() {
        this.#appClient = await AppWebsocket.connect(
            `ws://${this.#host}:${this.#appPort}`,
            30000,
            (signal) => signalHandler(self, signal))

        this.#appInfo = await this.#appClient.appInfo({ installed_app_id: this.#appId });
        this.#cellId = this.#appInfo.cell_data[0].cell_id;
        this.#agentPubKey = this.#cellId[1]
        this.#dna = this.#cellId[0]
        this.#dnaStr = bufferToBase64(this.#dna)
        // this.#agentPubKey = bufferToBase64(this.#agentPubKey);
        console.log("appinfo:{}", this.#appInfo);
    }

    async close() {
        // same, client.close() doesn't work in the browser
        await this.#appClient.client.socket.close();
    }

    async startNewGame(gameCode) {
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
    async makeMove(amount, prev_round_hash) {
        const params = {
            cap: null,
            cell_id: this.#cellId,
            zome_name: 'tragedy_of_commons',
            fn_name: 'make_new_move',
            provenance: this.#agentPubKey,
            payload: { resource_amount: amount, previous_round: prev_round_hash }
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
