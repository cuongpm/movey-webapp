function onSignIn(googleUser) {
    const isSignOut = getCookie('sign_out');
    if (isSignOut === 'true') {
        gapi.auth2.getAuthInstance().signOut();
        document.cookie = 'sign_out=; Path=/; Expires=Thu, 01 Jan 1970 00:00:01 GMT;'
        return
    }
    let profile = googleUser.getBasicProfile()
    window.location.href = `/accounts/google/callback?name=${profile.getName()}`
}

function getCookie(name) {
    const value = `; ${document.cookie}`;
    const parts = value.split(`; ${name}=`);
    if (parts.length === 2) return parts.pop().split(';').shift();
}