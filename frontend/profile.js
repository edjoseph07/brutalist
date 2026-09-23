document.addEventListener("DOMContentLoaded", () => {
    const usersPanel = document.getElementById("usersPanel");
    const aboutPanel = document.getElementById("aboutPanel");
    const usersList = document.getElementById("usersList");
    const usersStatus = document.getElementById("usersStatus");
    const loadMoreButton = document.getElementById("loadMoreButton");
    const navigationButtons = document.querySelectorAll(".sidebar-link");
    let currentPage = 0;

    function showPanel(panelId) {
        usersPanel.hidden = panelId !== "usersPanel";
        aboutPanel.hidden = panelId !== "aboutPanel";
        navigationButtons.forEach((button) => {
            button.classList.toggle("is-active", button.dataset.panel === panelId);
        });
    }

    async function signOut() {
        try {
            await fetch("http://127.0.0.1:3000/logout", {
    method: "POST",
    credentials: "include"
});
        } finally {
            localStorage.removeItem("token");
            window.location.replace("index.html");
        }
    }

    function appendUsers(users) {
        users.forEach((user) => {
            const card = document.createElement("article");
            card.className = "user-card";

            const name = document.createElement("h2");
            name.textContent = user.name;
            const email = document.createElement("p");
            email.className = "user-email";
            email.textContent = user.email;

            card.append(name, email);
            usersList.append(card);
        });
    }

    async function loadUsers(append = false) {
        const nextPage = append ? currentPage + 1 : 1;
        if (!append) {
            usersList.replaceChildren();
            usersStatus.textContent = "Loading users…";
        }
        loadMoreButton.disabled = true;

        try {
           const response = await fetch(
    `http://127.0.0.1:3000/users?page=${nextPage}`,
    {
        credentials: "include"
    }
);
            if (response.status === 401) {
                await signOut();
                return;
            }

            const result = await response.json().catch(() => ({}));
            if (!response.ok || !Array.isArray(result.users)) {
                throw new Error(result.message || "The server returned an invalid users response.");
            }

            currentPage = nextPage;
            appendUsers(result.users);
            const total = usersList.children.length;
            usersStatus.textContent = total === 0
                ? "No users have signed up yet."
                : `${total} user${total === 1 ? "" : "s"} loaded.`;
            loadMoreButton.hidden = result.has_more !== true;
        } catch (error) {
            console.error("Users request failed:", error);
            usersStatus.textContent = "Unable to load users. Please try again.";
        } finally {
            loadMoreButton.disabled = false;
        }
    }

    navigationButtons.forEach((button) => {
        button.addEventListener("click", () => showPanel(button.dataset.panel));
    });
    loadMoreButton.addEventListener("click", () => loadUsers(true));
    document.getElementById("logoutBtn").addEventListener("click", signOut);

    loadUsers();
});
