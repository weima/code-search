// Sample TypeScript file for testing call graph extraction

interface User {
    id: number;
    name: string;
    email: string;
}

class UserService {
    private users: User[] = [];

    public async createUser(userData: Omit<User, 'id'>): Promise<User> {
        const user = this.validateUserData(userData);
        const savedUser = await this.saveToDatabase(user);
        this.notifyUserCreated(savedUser);
        return savedUser;
    }

    private validateUserData(userData: Omit<User, 'id'>): User {
        if (!userData.name || !userData.email) {
            throw new Error('Invalid user data');
        }

        const isValidEmail = this.validateEmail(userData.email);
        if (!isValidEmail) {
            throw new Error('Invalid email format');
        }

        return {
            id: this.generateId(),
            ...userData
        };
    }

    private validateEmail(email: string): boolean {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(email);
    }

    private generateId(): number {
        return Math.max(...this.users.map(u => u.id), 0) + 1;
    }

    private async saveToDatabase(user: User): Promise<User> {
        // Simulate database save
        this.users.push(user);
        await this.delay(100);
        return user;
    }

    private notifyUserCreated(user: User): void {
        console.log(`User created: ${user.name} (${user.email})`);
        this.sendWelcomeEmail(user);
    }

    private async sendWelcomeEmail(user: User): Promise<void> {
        await this.delay(200);
        console.log(`Welcome email sent to ${user.email}`);
    }

    private delay(ms: number): Promise<void> {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    public async getUserById(id: number): Promise<User | null> {
        const user = this.findUserById(id);
        if (user) {
            this.logUserAccess(user);
        }
        return user;
    }

    private findUserById(id: number): User | null {
        return this.users.find(user => user.id === id) || null;
    }

    private logUserAccess(user: User): void {
        console.log(`User accessed: ${user.name}`);
    }
}

// Exported functions
export const createUserService = (): UserService => {
    return new UserService();
};

export function processUserBatch(users: Omit<User, 'id'>[]): Promise<User[]> {
    const service = createUserService();
    return Promise.all(users.map(userData => service.createUser(userData)));
}

// Arrow function exports
export const validateUserInput = (input: any): boolean => {
    return input && typeof input.name === 'string' && typeof input.email === 'string';
};

export const formatUser = (user: User): string => {
    return `${user.name} <${user.email}>`;
};

// Main execution
async function main(): Promise<void> {
    const service = createUserService();

    try {
        const newUser = await service.createUser({
            name: 'John Doe',
            email: 'john@example.com'
        });

        const retrievedUser = await service.getUserById(newUser.id);
        if (retrievedUser) {
            console.log(formatUser(retrievedUser));
        }
    } catch (error) {
        console.error('Error:', error);
    }
}

if (require.main === module) {
    main().catch(console.error);
}
