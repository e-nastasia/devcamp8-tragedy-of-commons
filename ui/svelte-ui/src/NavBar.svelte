<script>
    let Navbar;
</script>

<navbar>
    <nav class="container">
        <div class="item">
            <a href="/"><img alt="Logo" src="./logo-2x.png" height="35" /></a
            ><sup>community</sup>
        </div>
        <div class="item"/>
        <ul class="item">
            <li>
                <h2>Devcamp n°8 | 2021</h2>
            </li>
            <li><a href="https://forum.holochain.org/c/events/devcamp-8" target="_blank" rel="noopener noreferrer">Forum</a></li>
            <li>
                <a href="#">Documentation</a>
                <ul>
                    <li><a href="https://developer.holochain.org/" target="_blank" rel="noopener noreferrer">Getting started</a></li>
                    <li><a href="https://developer.holochain.org/resources/" target="_blank" rel="noopener noreferrer">Resources</a></li>
                </ul>
            </li>
        </ul>
    </nav>
</navbar>

<style>
    nav{
        margin-bottom: 0px;
    }
    .container {
        display: flex;
        justify-content: space-around;
        /* by default: align-items: strech -> child elements' heights = the container's height */
    }
</style>
