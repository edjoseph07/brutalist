document.addEventListener("DOMContentLoaded", () => {
    const loginForm = document.getElementById("loginForm");
    const emailInput = document.getElementById("email");
    const passwordInput = document.getElementById("password");
    const emailError = document.getElementById("emailError");
    const passwordError = document.getElementById("passwordError");
    const passwordToggle = document.getElementById("passwordToggle");

    function setFieldError(element, message) {
        element.textContent = message;
        element.classList.add("show");
        element.closest(".form-group")?.classList.add("error");
    }

    function clearFieldError(element) {
        element.textContent = "";
        element.classList.remove("show");
        element.closest(".form-group")?.classList.remove("error");
    }

    if (passwordToggle) {
        const toggleText = passwordToggle.querySelector(".toggle-text");
        passwordToggle.addEventListener("click", () => {
            const isHidden = passwordInput.type === "password";
            passwordInput.type = isHidden ? "text" : "password";
            if (toggleText) {
                toggleText.textContent = isHidden ? "HIDE" : "SHOW";
            }
        });
    }

    loginForm.addEventListener("submit", async (event) => {
        event.preventDefault();
        clearFieldError(emailError);
        clearFieldError(passwordError);

        const email = emailInput.value.trim();
        const password = passwordInput.value;
        let hasError = false;

        if (!/^\S+@\S+\.\S+$/.test(email)) {
            setFieldError(emailError, "Enter a valid email address.");
            hasError = true;
        }

        if (!password) {
            setFieldError(passwordError, "Password is required.");
            hasError = true;
        }

        if (hasError) {
            return;
        }

        try {
            const response = await fetch("http://127.0.0.1:3000/login", {
    method: "POST",
    headers: {
        "Content-Type": "application/json"
    },
    credentials: "include",
    body: JSON.stringify({ email, password }),
});
            const result = await response.json().catch(() => ({}));

            if (!response.ok) {
                setFieldError(passwordError, result.message || "Unable to sign in.");
                return;
            }

            window.location.assign("profile.html");
        } catch (error) {
            console.error("Login error:", error);
            setFieldError(passwordError, "Unable to connect to the server.");
        }
    });
});
