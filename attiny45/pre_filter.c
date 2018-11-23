#define F_CPU 8000000UL

#include <stdint.h>
#include <avr/io.h>
#include <avr/interrupt.h>
#include <avr/sleep.h>
#include <avr/power.h>
#include <util/delay.h>

//out PB3
//in PB4

volatile uint8_t send_val;

void init_system(void){
        cli();

        //DDRB  = 0xFF ^ (1<<PB4); //1 = Ausgang, 0 = Eingang
        DDRB = 1<<PB3;
        PORTB |= (1<<PB3); //set H

        PCMSK |= 1<<PCINT4; //enabled pin change interrupt
        GIMSK |= 1<<PCIE;// Pin Change Interrupt Enable
        MCUCR |= 1<<ISC00;

        // Timer 0 konfigurieren
        TCCR0A = (1<<WGM01); // CTC Modus
        TCCR0B |= (1<<CS01); // Prescaler 8 -> F_CPU/8 -> 1us

        OCR0A = 100-1;//in us

        // Compare Interrupt erlauben
        TIMSK |= (1<<OCIE0A);

        send_val = 1;

        sei();

        power_adc_disable();
        power_usi_disable();
        power_timer1_disable();

        set_sleep_mode(SLEEP_MODE_IDLE);
}

int main (void)
{
        init_system();

        while(1) {
                //sleep_mode();
                asm volatile("NOP");
/*
//check IO: ok
                _delay_ms(500);
           if (PINB & (1<<PB4))
                PORTB |= 1<<PB3;
           else
                PORTB &= ~(1<<PB3);*/
        }
}

ISR(TIMER0_COMPA_vect){ //all 1ms - ok
        send_val = 1;
}

ISR(PCINT0_vect){
        cli();
        if (PINB & (1<<PB4)) {
                //pin now H -> only follow if pulse is long enought
                if (send_val) {
                        PORTB |= 1<<PB3;
                }
        }else{
                //pin now low -> do so as well
                PORTB &= ~(1<<PB3);
                send_val = 0;
        }
        sei();
}
