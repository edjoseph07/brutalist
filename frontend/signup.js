document.addEventListener("DOMContentLoaded", () => {
    const signupForm = document.getElementById("signupForm");
    const nameInput = document.getElementById("name");
    const emailInput = document.getElementById("email");
    const passwordInput = document.getElementById("password");
    const passwordToggle = document.getElementById("passwordToggle");
    const nameError = document.getElementById("nameError");
    const emailError = document.getElementById("emailError");
    const passwordError = document.getElementById("passwordError");
    const message = document.getElementById("message");

    function setFieldError(element, text) {
        element.textContent = text;
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

    signupForm.addEventListener("submit", async (event) => {
        event.preventDefault();
        [nameError, emailError, passwordError].forEach(clearFieldError);
        message.textContent = "";
        message.className = "form-message";

        const name = nameInput.value.trim();
        const email = emailInput.value.trim();
        const password = passwordInput.value;
        let hasError = false;

        if (!name || name.length > 100) {
            setFieldError(nameError, "Name must be between 1 and 100 characters.");
            hasError = true;
        }

        if (!/^\S+@\S+\.\S+$/.test(email)) {
            setFieldError(emailError, "Enter a valid email address.");
            hasError = true;
        }

        if (password.length < 8 || password.length > 128) {
            setFieldError(passwordError, "Password must be between 8 and 128 characters.");
            hasError = true;
        }

        if (hasError) {
            return;
        }

        const data = {
    name: name,
    email: email,
    password: password
};
        try {
            const response = await fetch("http://127.0.0.1:3000/signup", {
                method: "POST",
                    headers: {
                "Content-Type": "application/json"
    },
    body: JSON.stringify(data)
});
            const result = await response.json().catch(() => ({}));

            message.textContent = result.message || "Unable to complete sign-up.";
            message.className = `form-message ${response.ok ? "success" : "error"}`;

            if (response.ok) {
                signupForm.reset();
                setTimeout(() => window.location.assign("index.html"), 1000);
            }
        } catch (error) {
            console.error("Signup error:", error);
            message.textContent = "Unable to connect to the server.";
            message.className = "form-message error";
        }
    });
});
