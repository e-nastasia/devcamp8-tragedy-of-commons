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
            <li><a href="#">Forum</a></li>
            <li>
                <a href="#">Documentation</a>
                <ul>
                    <li><a href="#">Sublink with a long name</a></li>
                    <li><a href="#">Short sublink</a></li>
                </ul>
            </li>
        </ul>
    </nav>
</navbar>

<style>
    .container {
        display: flex;
        justify-content: space-around;
        /* by default: align-items: strech -> child elements' heights = the container's height */
    }
</style>
