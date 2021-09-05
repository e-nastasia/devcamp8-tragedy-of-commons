import { AdminWebsocket, AppWebsocket } from '@holochain/conductor-api'
import { bufferToBase64, decodeJson, encodeJson } from './utils.js'

export class Connection {
    constructor(appHost, appPort, appId) {
        this.appHost = appHost;
        this.appPort = appPort;
        this.appId = appId;
        this.sessions = []
    }

    async open() {
        var self = this;
        this.appClient = await AppWebsocket.connect(
            `ws://${this.appHost}:${this.appPort}`,
            30000,
            (signal) => signalHandler(self, signal))

        console.log('connection established:', this)

        // TODO: in the future we should be able manage and to attach to multiple syn happs
        //   this.syn = new Syn(defaultContent, applyDeltaFn, this.appClient, this.appId)
        //   await this.syn.attach(this.appId)
        //   this.sessions = await this.syn.getSessions()
    }
}

function signalHandler(connection, signal) {
    // ignore signals not meant for me
    // if (!connection.syn || bufferToBase64(signal.data.cellId[1]) != connection.syn.me) {
    //   return
    // }
    switch (signal.data.payload.signal_name) {
        case 'CommitNotice':
        //connection.session.commitNotice(signal.data.payload.signal_payload)
    }
}

export class Zome {
    constructor(appClient, appId) {
        this.appClient = appClient
        this.appId = appId
    }

    async attach() {
        // setup the syn instance data
        this.appInfo = await this.appClient.appInfo({ installed_app_id: this.appId });
        this.cellId = this.appInfo.cell_data[0].cell_id
        this.agentPubKey = this.cellId[1]
        this.dna = this.cellId[0]
        this.dnaStr = bufferToBase64(this.dna)
        this.me = bufferToBase64(this.agentPubKey);
        console.log("appinfo:{}", this.appInfo);
    }

    attached() {
        return this.appInfo != undefined
    }

    async call(fn_name, payload, timeout) {
        if (!this.attached()) {
            console.log("Can't call zome when disconnected from conductor")
            return
        }
        try {
            const zome_name = 'syn'
            console.log(`Making zome call ${fn_name} with:`, payload)
            const result = await this.appClient.callZome(
                {
                    cap: null,
                    cell_id: this.cellId,
                    zome_name,
                    fn_name,
                    provenance: this.agentPubKey,
                    payload
                },
                timeout
            )
            return result
        } catch (error) {
            console.log('ERROR: callZome threw error', error)
            throw (error)
            //  if (error == 'Error: Socket is not open') {
            // TODO        return doResetConnection(dispatch)
            // }
        }
    }

}

