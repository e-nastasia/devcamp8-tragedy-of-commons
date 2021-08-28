<script>
	import NavBar from "./NavBar.svelte";
	import StartMenu from "./StartMenu.svelte";
	import Game from "./Game.svelte";
</script>

<NavBar />

<!-- <StartMenu/> -->
<Game/>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 240px;
		margin: 0 auto;
	}


	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
</style>
