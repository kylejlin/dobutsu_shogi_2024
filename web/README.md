# Dobutsu Shogi Analyzer Web App

## Running

```sh
npm install
npm start
```

## A note about cross-origin requests

The app relies on a database hosted on the `kylejlin.github.io` domain.
At the time of writing, I am able to make cross-origin requests to this domain
from `localhost`, which is good enough for development purposes.
However, depending on how GitHub's rules change, and how browsers rules surrounding `localhost` change,
this may break in the future.

If you run into this issue, you can either:

- Disable CORS in your browser.
  This is very dangerous, so you better know what you're doing.

  I used [this](https://addons.mozilla.org/en-US/firefox/addon/cors-everywhere/) Firefox extension to do this.

- Clone the database and then change the URLs in `src/App.tsx` to point to your local copy.

  If you grep for `kylejlin.github.io`, you should find all the URLs you need to change.

  The database is just a big GitHub repository, so a normal `git clone` should work.

  However, the database is quite large, so it may take a while to clone.
  If your network is not stable enough to support the clone,
  you can try building the database from scratch.

  To build the database from scratch, first run

  ```sh
  cargo run --release
  ```

  After several hours (or days, depending on your computer's speed), you should see a prompt that says something like

  ```txt
  Tree inspector ready. Type \"launch\" to launch or \"simpledb\" to create a simple best-child database.
  Launching will clear the console, so be sure to save any important information.
  ```

  Once you see this prompt, type `simpledb` and press Enter.
  This should create a new database in a directory named `db`.
  It may take several hours (or even days).

  At this point, it's up to you to figure out how to serve the database.
  You can serve it locally, use GitHub Pages, etc.
