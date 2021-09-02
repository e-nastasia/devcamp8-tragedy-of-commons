import { AdminWebsocket, AppWebsocket } from '@holochain/conductor-api';

const HOST = "localhost";
const APP_PORT = 8888;
const ADMIN_PORT = 60386;

export class AppClient {
    #host = '';
    #adminPort = '';
    #appPort = '';
    #appClient = null;
    #cellIds = [];
    #agentPubKey = '';

    constructor() {
        this.#host = HOST || 'localhost';
        this.#adminPort = ADMIN_PORT;
        this.#appPort = APP_PORT || '8888';
    }
    async connect() {
        const adminClient = await AdminWebsocket.connect(`ws://${this.#host}:${this.#adminPort}`);
        this.#cellIds = await adminClient.listCellIds();
        this.#agentPubKey = this.#cellIds[0][1];
        // doesn't work in the browser
        // adminClient.client.close();
        adminClient.client.socket.close();
        this.#appClient = await AppWebsocket.connect(`ws://${this.#host}:${this.#appPort}`);
    }

    async close() {
        // same, client.close() doesn't work in the browser
        await this.#appClient.client.socket.close();
    }

    async startNewGame(gameCode) {
        const params = {
            cap: null,
            cell_id: this.#cellIds[0],
            zome_name: 'tragedy_of_commons',
            fn_name: 'create_game_code_anchor',
            provenance: this.#agentPubKey,
            payload: gameCode
        };
        return this.#appClient.callZome(params);
    }
}