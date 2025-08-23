type BodyStyle = 'Sedan' | 'Hatchback' | 'Wagon' | 'SUV' | 'Pickup';

interface Car {
    make: string;
    model: string;
    year: number;
    bodyStyle: BodyStyle;
}

interface SportsCar extends Car {
    sportMode: boolean;
    spoiler: boolean;
}

interface Truck extends Car {
    towingCapacity: number;
}

const car: Car = {
    make: 'Honda',
    model: 'Civic',
    year: 2022,
    bodyStyle: 'Sedan'
};
const sportsCar: SportsCar = {
    make: 'SAAB',
    model: '9-5 Aero',
    year: 2005,
    sportMode: true,
    spoiler: true,
    bodyStyle: 'Wagon'
};

const truck: Truck = {
    make: 'Toyota',
    model: 'Tacoma',
    year: 2016,
    towingCapacity: 3500,
    bodyStyle: 'Pickup'
};

console.log(car);
console.log(sportsCar);
console.log(truck);
