---
date: 09/13/2025
title: Fix 404 Error on Github Pages After Configuring Domain
blurb: Detail of a weird error I encountered configuring a hostname for this blog
---

# TLDR
- If you're running Github Actions, and configured the domain successfully - but still getting 404 errors - you'll have to rebuild the site again, OR refresh your browser cache.
- `git commit --allow-empty -m "Trigger rebuild"`
- then push to your remote. This will solve your issue
- If it doesn't work, make sure `index.html` is in your artifact at the root level. 
--- 

I recently decided to configure a hostname for this blog! Something to make this a bit more personal to me. I decided going with mavwrites.com - mainly because maverickwrites.com was taken - but also because it's a bit of a mouthful. mavwrites is short and succinct. 

I bought my hostname off PorkBun, a domain registry company that makes it easy to register domains - and I think even do webhosting and email hosting. I think in the future I would like to try to set up email hosting.

I first bought the domain, then went to verify it on Github, which you can do by going to the profile settings and look for the Pages tab. There you can issue a challenge to the DNS record to verify the domain to your account. It's highly recommended that you do this.

Then I went to my blog repo and added in the custom domain, then enabled HTTPS. The verification went through - https initialization finished - and I went to the newly published `mavwrites.com`. However I ran into a 404 error! Which was odd. I was only able to access the root after appending `/index.html`.

I checked the documentation to see if the rules on `index.html` changed, made sure my artifact has `index.html` at the root, but nothing happened.

My blog has a CI/CD pipeline that compiles all of the `.md` files within the `content` directory and converts them to static HTML files. According to the Github documentation, there's no need to provide a CNAME file if you provide a pipeline. So all should've been good.

I was able to solve it by triggering an empty rebuild to the pipeline, allowing Github to update it's root for me. 

On further reflection - it may have to do with the browser cache not updating, and I probably should have tried to cleared the cache and refresh the browser page.