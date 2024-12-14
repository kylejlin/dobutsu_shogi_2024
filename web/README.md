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

- Clone the database (it's just an ordinary Git repository, albeit one with more files than usual) and then change the URLs in `src/App.tsx` to point to your local copy.
  If you grep for `kylejlin.github.io`, you should find all the URLs you need to change.
