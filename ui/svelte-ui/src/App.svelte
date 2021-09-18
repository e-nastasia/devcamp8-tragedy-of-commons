<script>
	import NavBar from "./NavBar.svelte";
	import StartMenu from "./StartMenu.svelte";
	import Game from "./Game.svelte";
	import { AppClient } from "./app-client";
	import { onMount, onDestroy } from 'svelte';

	let status = "START"; // "GAME_BEGIN"  "GAME_JOIN" "LOADING"
	let nickname = "---";
	let gamecode = "------";
	let errorMessage = "";

	async function startNewGame(event) {
		if (status === "LOADING") {
			// prevent multiple clicks
			return;
		}
		nickname = event.detail.nickname;
		try {
			status = "LOADING";
			const anchor = await window.appClient.startNewGame();
			console.log("anchor", anchor);
			const result = await window.appClient.joinGame(gamecode,nickname);
			console.log("joined game", result);
			status = "GAME_BEGIN";
		} catch (error) {
			errorMessage = error.data?.data || error.message;
			console.log("error", error);
		}
	}

	async function joinGame(event) {
		if (status === "LOADING") {
			return;
		}
		status = "LOADING";
		nickname = event.detail.nickname;
		gamecode = event.detail.gamecode;
		console.log("nick and code: ", nickname, gamecode)
		console.log("gamecode", gamecode)
		const result = await window.appClient.joinGame(gamecode, nickname);
		console.log("joingame",result);
		status = "GAME_JOIN";
	}
	/****************************************/
	let appHost = "localhost";
	let appPort = 8000;

	async function connect() {
		const appClient = new AppClient(appHost, appPort);
		try {
			await appClient.connect();
			window.appClient = appClient;
			errorMessage  = '';
		} catch (error) {
			errorMessage = error.data?.data || error.message;
		}
	}

	onMount(connect);

	onDestroy(() => {
		if (window.appClient) {
			window.appClient.close();
		}
	})
</script>

<NavBar />

{#if status === "START"}
	<StartMenu on:startNewGame={startNewGame} on:joinGame={joinGame} />
{:else if status == "LOADING"}
	<div style="flex-flow:column; text-align: center;">
		<h1>LOADING...</h1>
		<p>Connecting to conductor...</p>
		<p>Calling zome...</p>
	</div>
{:else}
	<Game action={status} {nickname} {gamecode} />
{/if}

<footer>
	{#if errorMessage}
		<h3 style="color: red;">{errorMessage}</h3>
	{/if}
</footer>

<style>
	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}
</style>
